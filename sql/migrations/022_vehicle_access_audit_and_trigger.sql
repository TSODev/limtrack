-- Migration 022 : aligne le dépôt sur le schéma réel de production, découvert désynchronisé
-- en réappliquant seed_appstore_review.sql sur la prod (2026-09-16) : deux changements avaient
-- été faits manuellement sur le VPS à un moment non documenté, jamais capturés en migration ni
-- reportés sur la base de dev locale.
--
-- 1. Colonnes d'audit sur vehicle_access — même convention que fleet_roles.granted_by
--    (fleet_migration.sql) : qui a accordé l'accès, et quand.
ALTER TABLE public.vehicle_access
    ADD COLUMN IF NOT EXISTS granted_by UUID REFERENCES public.users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE public.vehicle_access
    ALTER COLUMN role SET DEFAULT 'viewer';

-- 2. Trigger auto_grant_owner — accorde automatiquement le rôle 'owner' sur vehicle_access
--    à l'insertion d'un véhicule. Sert de filet de sécurité pour toute insertion SQL directe
--    (seeds, scripts d'import, futures migrations de données) qui oublierait d'insérer
--    vehicle_access elle-même — complète (ne remplace pas) l'insert explicite déjà fait par
--    vehicles_handler.rs::create_vehicle dans la même transaction (migration 019 : ce
--    correctif applicatif reste nécessaire, le trigger ne couvre que les insertions SQL qui
--    contournent l'API).
CREATE OR REPLACE FUNCTION public.auto_grant_owner()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    INSERT INTO public.vehicle_access (vehicle_id, user_id, role, granted_by)
    VALUES (NEW.id, NEW.owner_id, 'owner', NEW.owner_id)
    ON CONFLICT (vehicle_id, user_id) DO NOTHING;
    RETURN NEW;
END;
$function$;

DROP TRIGGER IF EXISTS trg_auto_grant_owner ON public.vehicles;
CREATE TRIGGER trg_auto_grant_owner
    AFTER INSERT ON public.vehicles
    FOR EACH ROW EXECUTE FUNCTION public.auto_grant_owner();
