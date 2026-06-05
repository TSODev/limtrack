-- =============================================================================
-- JEUX DE DONNÉES APP STORE REVIEW — Compte de démonstration iOS
-- =============================================================================
-- Prérequis : pgcrypto activée sur NeonDB
--   CREATE EXTENSION IF NOT EXISTS pgcrypto;
--
-- Importer avec psql :
--   psql "$DATABASE_URL" -f seed_appstore_review.sql
--
-- Comptes créés :
--   apple.reviewer / AppReview2024!  → compte iOS principal (is_ios=true, accès lifetime)
--   demo.friend    / DemoFriend2024! → second compte pour illustrer le partage de véhicule
--
-- Véhicules — cas d'usage couverts :
--   AR-001-AA  Renault Clio        — LOA saine (55% km), assurance saine           ✅
--   AR-002-BB  Volkswagen Golf     — LOA à 87% km → alerte dépassement proche      ⚠️
--   AR-003-CC  Peugeot 208         — LOA expire dans 28 jours → alerte date        ⚠️
--   AR-004-DD  Toyota C-HR         — LOA expirée + km dépassé + assurance expirée  ❌
--   AR-005-EE  Citroën C3 Aircross — partagé par demo.friend (rôle viewer)         👁
--
-- Fonctionnalités illustrées :
--   - Dashboard véhicule avec trajectoire idéale et jalons kilométriques
--   - Contrat LOA : dates, km alloués, coût au km supplémentaire (price_per_extra_km)
--   - Contrat assurance : assureur, limite annuelle, renouvellement
--   - Historique de relevés kilométriques (6 à 16 entrées par véhicule)
--   - États d'alerte : vert / orange / rouge
--   - Partage de véhicule (vue partagée en lecture seule)
--   - Profil utilisateur et préférences de notifications
-- =============================================================================

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- =============================================================================
-- NETTOYAGE (idempotent — ré-exécutable sans erreur)
-- =============================================================================

DELETE FROM public.mileage_log WHERE vehicle_id IN (
    SELECT id FROM public.vehicles WHERE plate_number LIKE 'AR-%');
DELETE FROM public.contracts_insurance WHERE vehicle_id IN (
    SELECT id FROM public.vehicles WHERE plate_number LIKE 'AR-%');
DELETE FROM public.contracts_loa WHERE vehicle_id IN (
    SELECT id FROM public.vehicles WHERE plate_number LIKE 'AR-%');
DELETE FROM public.vehicle_access WHERE vehicle_id IN (
    SELECT id FROM public.vehicles WHERE plate_number LIKE 'AR-%');
DELETE FROM public.vehicles WHERE plate_number LIKE 'AR-%';
DELETE FROM public.user_preferences WHERE user_id IN (
    'f0000000-0000-4000-8000-aaaaaa000001'::uuid,
    'f0000000-0000-4000-8000-aaaaaa000002'::uuid
);
DELETE FROM public.users WHERE id IN (
    'f0000000-0000-4000-8000-aaaaaa000001'::uuid,
    'f0000000-0000-4000-8000-aaaaaa000002'::uuid
);

-- =============================================================================
-- UTILISATEURS
-- =============================================================================

INSERT INTO public.users
    (id, username, email, password_hash, trial_ends_at, access_expires_at, is_ios)
VALUES
    -- apple.reviewer — compte iOS App Store (lifetime, is_ios=true → Flotte masquée)
    (
        'f0000000-0000-4000-8000-aaaaaa000001',
        'apple.reviewer',
        'appstore-review@limtrack.app',
        crypt('AppReview2024!', gen_salt('bf', 12)),
        '2099-01-01 00:00:00+00',
        '2099-01-01 00:00:00+00',
        true
    ),
    -- demo.friend — second compte pour le scénario de partage
    (
        'f0000000-0000-4000-8000-aaaaaa000002',
        'demo.friend',
        'demo-friend@limtrack.app',
        crypt('DemoFriend2024!', gen_salt('bf', 12)),
        '2099-01-01 00:00:00+00',
        '2099-01-01 00:00:00+00',
        false
    );

-- Préférences de notifications
INSERT INTO public.user_preferences (user_id, notif_days_before, notif_km_percent, updated_once)
VALUES
    ('f0000000-0000-4000-8000-aaaaaa000001', 30, 10, true),
    ('f0000000-0000-4000-8000-aaaaaa000002', 15, 15, true)
ON CONFLICT (user_id) DO NOTHING;

-- =============================================================================
-- VÉHICULES
-- =============================================================================
-- company_id / org_id intentionnellement NULL : usage personnel (is_ios)

INSERT INTO public.vehicles (id, owner_id, make, model, plate_number, year, vin)
VALUES
    -- v1 Renault Clio — LOA saine ✅
    (
        'f1111111-0000-4000-8000-aaaaaa000001',
        'f0000000-0000-4000-8000-aaaaaa000001',
        'Renault', 'Clio V', 'AR-001-AA', 2022,
        'VF1BJA00060000001'
    ),
    -- v2 Volkswagen Golf — 87% km consommé ⚠️
    (
        'f1111111-0000-4000-8000-aaaaaa000002',
        'f0000000-0000-4000-8000-aaaaaa000001',
        'Volkswagen', 'Golf VIII', 'AR-002-BB', 2022,
        'WVWZZZ1KZMW000002'
    ),
    -- v3 Peugeot 208 — LOA expire dans 28 jours ⚠️
    (
        'f1111111-0000-4000-8000-aaaaaa000003',
        'f0000000-0000-4000-8000-aaaaaa000001',
        'Peugeot', '208', 'AR-003-CC', 2021,
        'VF3CCBHZ0MT000003'
    ),
    -- v4 Toyota C-HR — LOA expirée + km dépassé + assurance expirée ❌
    (
        'f1111111-0000-4000-8000-aaaaaa000004',
        'f0000000-0000-4000-8000-aaaaaa000001',
        'Toyota', 'C-HR', 'AR-004-DD', 2022,
        'NMTK33BV40R000004'
    ),
    -- v5 Citroën C3 — appartient à demo.friend, partagé en viewer avec apple.reviewer
    (
        'f1111111-0000-4000-8000-aaaaaa000005',
        'f0000000-0000-4000-8000-aaaaaa000002',
        'Citroën', 'C3 Aircross', 'AR-005-EE', 2023,
        'VF7SXBHY0NT000005'
    );

-- =============================================================================
-- ACCÈS VÉHICULES
-- =============================================================================
-- Les accès 'owner' sont créés automatiquement par le trigger trg_auto_grant_owner.
-- On insère uniquement le partage viewer : demo.friend → apple.reviewer sur la C3.

INSERT INTO public.vehicle_access (vehicle_id, user_id, role)
VALUES (
    'f1111111-0000-4000-8000-aaaaaa000005',
    'f0000000-0000-4000-8000-aaaaaa000001',
    'viewer'
)
ON CONFLICT (vehicle_id, user_id) DO NOTHING;

-- =============================================================================
-- CONTRATS LOA
-- =============================================================================

INSERT INTO public.contracts_loa
    (id, vehicle_id, km_allowed, km_start, start_date, end_date, price_per_extra_km)
VALUES
    -- v1 Clio — 36 mois, 30 000 km, km_consumed=16 500 (55%) ✅
    (
        'f2222222-0000-4000-8000-aaaaaa000001',
        'f1111111-0000-4000-8000-aaaaaa000001',
        30000, 12000,
        '2024-01-01', '2027-01-01',
        0.18
    ),
    -- v2 Golf — 48 mois, 60 000 km, km_consumed=52 500 (87.5%) ⚠️
    (
        'f2222222-0000-4000-8000-aaaaaa000002',
        'f1111111-0000-4000-8000-aaaaaa000002',
        60000, 8000,
        '2022-10-01', '2026-10-01',
        0.22
    ),
    -- v3 Peugeot 208 — 36 mois, expire 2026-07-01 (J-26), km_consumed=24 500 (68%)
    (
        'f2222222-0000-4000-8000-aaaaaa000003',
        'f1111111-0000-4000-8000-aaaaaa000003',
        36000, 3500,
        '2023-07-01', '2026-07-01',
        0.15
    ),
    -- v4 Toyota C-HR — expirée 2026-02-01, km_consumed=46 500 (103%) → dépassement 1 500 km ❌
    (
        'f2222222-0000-4000-8000-aaaaaa000004',
        'f1111111-0000-4000-8000-aaaaaa000004',
        45000, 15000,
        '2023-02-01', '2026-02-01',
        0.20
    ),
    -- v5 Citroën C3 — 36 mois, km_consumed=5 500 (18%) ✅
    (
        'f2222222-0000-4000-8000-aaaaaa000005',
        'f1111111-0000-4000-8000-aaaaaa000005',
        30000, 22000,
        '2024-09-01', '2027-09-01',
        NULL
    );

-- =============================================================================
-- CONTRATS ASSURANCE
-- =============================================================================

INSERT INTO public.contracts_insurance
    (id, vehicle_id, km_annual_limit, km_start, start_date, end_date, insurer)
VALUES
    -- v1 Clio — AXA, renouvelée jan. 2026, saine
    (
        'f3333333-0000-4000-8000-aaaaaa000001',
        'f1111111-0000-4000-8000-aaaaaa000001',
        12000, 20000,
        '2026-01-01', '2027-01-01',
        'AXA'
    ),
    -- v2 Golf — Groupama, renouvelée jan. 2026, saine
    (
        'f3333333-0000-4000-8000-aaaaaa000002',
        'f1111111-0000-4000-8000-aaaaaa000002',
        18000, 55000,
        '2026-01-01', '2027-01-01',
        'Groupama'
    ),
    -- v3 Peugeot 208 — MAAF, renouvelée jan. 2026, saine
    (
        'f3333333-0000-4000-8000-aaaaaa000003',
        'f1111111-0000-4000-8000-aaaaaa000003',
        12000, 24000,
        '2026-01-01', '2027-01-01',
        'MAAF'
    ),
    -- v4 Toyota C-HR — MMA, expirée 2026-02-01 ❌
    (
        'f3333333-0000-4000-8000-aaaaaa000004',
        'f1111111-0000-4000-8000-aaaaaa000004',
        15000, 46000,
        '2025-02-01', '2026-02-01',
        'MMA'
    ),
    -- v5 Citroën C3 — AXA, saine
    (
        'f3333333-0000-4000-8000-aaaaaa000005',
        'f1111111-0000-4000-8000-aaaaaa000005',
        12000, 23000,
        '2025-09-01', '2026-09-01',
        'AXA'
    );

-- =============================================================================
-- RELEVÉS KILOMÉTRIQUES
-- =============================================================================

-- ─── v1 Renault Clio (final = 28 500 — LOA 55% ✅) ───────────────────────────
-- Insurance démarre 2026-01-01 → contract_insurance_id NULL avant cette date.
INSERT INTO public.mileage_log
    (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
VALUES
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    13500, '2024-02-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    16800, '2024-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    20200, '2024-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    22900, '2025-02-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    25500, '2025-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', NULL,                                    27300, '2025-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', 'f3333333-0000-4000-8000-aaaaaa000001', 27800, '2026-01-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000001', 'f2222222-0000-4000-8000-aaaaaa000001', 'f3333333-0000-4000-8000-aaaaaa000001', 28500, '2026-04-01', 'manual');

-- ─── v2 Volkswagen Golf (final = 60 500 — LOA 87.5% ⚠️) ─────────────────────
INSERT INTO public.mileage_log
    (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
VALUES
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    10500, '2022-12-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    16000, '2023-03-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    22500, '2023-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    29000, '2023-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    35500, '2023-12-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    41000, '2024-03-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    46500, '2024-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    50500, '2024-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    54000, '2024-12-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    56500, '2025-03-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    58000, '2025-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    59000, '2025-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', NULL,                                    59800, '2025-12-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', 'f3333333-0000-4000-8000-aaaaaa000002', 60000, '2026-01-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', 'f3333333-0000-4000-8000-aaaaaa000002', 60200, '2026-04-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000002', 'f2222222-0000-4000-8000-aaaaaa000002', 'f3333333-0000-4000-8000-aaaaaa000002', 60500, '2026-06-01', 'manual');

-- ─── v3 Peugeot 208 (final = 28 000 — LOA J-26 ⚠️) ──────────────────────────
INSERT INTO public.mileage_log
    (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
VALUES
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                     5200, '2023-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                     8500, '2024-01-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    12000, '2024-05-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    15500, '2024-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    18800, '2025-01-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    21900, '2025-05-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    24700, '2025-09-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', NULL,                                    26500, '2025-12-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', 'f3333333-0000-4000-8000-aaaaaa000003', 26800, '2026-01-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', 'f3333333-0000-4000-8000-aaaaaa000003', 27500, '2026-03-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000003', 'f2222222-0000-4000-8000-aaaaaa000003', 'f3333333-0000-4000-8000-aaaaaa000003', 28000, '2026-06-01', 'manual');

-- ─── v4 Toyota C-HR (final = 61 500 — LOA expirée + 1 500 km dépassés ❌) ────
-- LOA expirée 2026-02-01, assurance expirée 2026-02-01.
-- Coût dépassement : 1 500 km × 0.20 €/km = 300 €
INSERT INTO public.mileage_log
    (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
VALUES
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    17000, '2023-03-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    21500, '2023-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    27000, '2023-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    33000, '2024-02-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    38500, '2024-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', NULL,                                    44000, '2024-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', 'f3333333-0000-4000-8000-aaaaaa000004', 48000, '2025-02-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', 'f3333333-0000-4000-8000-aaaaaa000004', 53500, '2025-06-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', 'f3333333-0000-4000-8000-aaaaaa000004', 58500, '2025-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', 'f3333333-0000-4000-8000-aaaaaa000004', 61000, '2026-01-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000004', 'f2222222-0000-4000-8000-aaaaaa000004', 'f3333333-0000-4000-8000-aaaaaa000004', 61500, '2026-03-01', 'manual');

-- ─── v5 Citroën C3 Aircross (final = 27 500 — sain 18% ✅) ───────────────────
-- Appartient à demo.friend, partagé en viewer avec apple.reviewer.
-- Insurance démarre 2025-09-01.
INSERT INTO public.mileage_log
    (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
VALUES
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', NULL,                                    23200, '2024-10-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', NULL,                                    24500, '2025-01-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', NULL,                                    25800, '2025-04-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', 'f3333333-0000-4000-8000-aaaaaa000005', 26500, '2025-09-15', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', 'f3333333-0000-4000-8000-aaaaaa000005', 27000, '2026-01-01', 'manual'),
    ('f1111111-0000-4000-8000-aaaaaa000005', 'f2222222-0000-4000-8000-aaaaaa000005', 'f3333333-0000-4000-8000-aaaaaa000005', 27500, '2026-05-01', 'manual');

COMMIT;

-- =============================================================================
-- VÉRIFICATION RAPIDE (optionnel — à exécuter séparément)
-- =============================================================================
-- SELECT u.username, v.make, v.model, v.plate_number,
--        cl.km_allowed, cl.km_start,
--        ROUND(((ml.value - cl.km_start)::numeric / cl.km_allowed) * 100, 1) AS pct_km,
--        cl.start_date, cl.end_date,
--        ci.insurer,
--        ml.value AS km_actuel
-- FROM public.users u
-- JOIN public.vehicles v ON v.owner_id = u.id
-- JOIN public.contracts_loa cl ON cl.vehicle_id = v.id
-- LEFT JOIN public.contracts_insurance ci ON ci.vehicle_id = v.id
-- JOIN LATERAL (
--   SELECT value FROM public.mileage_log
--   WHERE vehicle_id = v.id
--   ORDER BY recorded_at DESC LIMIT 1
-- ) ml ON true
-- WHERE v.plate_number LIKE 'AR-%'
-- ORDER BY u.username, v.plate_number;
