-- Migration 012 : vue v_contract_status
-- Centralise le calcul du statut des contrats (danger/warning/ok)
-- pour éviter la duplication entre vehicles_handler.rs et contracts_handler.rs.
-- Utiliser cette vue partout où le statut agrégé d'un véhicule est nécessaire.

CREATE OR REPLACE VIEW public.v_contract_status AS
SELECT
    v.id AS vehicle_id,
    CASE
        -- danger : km consommés >= km autorisés (LOA ou assurance)
        WHEN EXISTS (
            SELECT 1 FROM public.contracts_loa l
            WHERE l.vehicle_id = v.id
              AND COALESCE(
                  (SELECT value FROM public.mileage_log
                   WHERE vehicle_id = v.id
                   ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                  l.km_start
              ) - l.km_start >= l.km_allowed
        ) OR EXISTS (
            SELECT 1 FROM public.contracts_insurance i
            WHERE i.vehicle_id = v.id
              AND COALESCE(
                  (SELECT value FROM public.mileage_log
                   WHERE vehicle_id = v.id
                   ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                  i.km_start
              ) - i.km_start >= i.km_annual_limit
        ) THEN 'danger'

        -- warning : contrat actif expirant dans ≤30j OU projection km dépasse le plafond
        WHEN EXISTS (
            SELECT 1 FROM public.contracts_loa l
            WHERE l.vehicle_id = v.id
              AND l.end_date >= CURRENT_DATE
              AND (
                  l.end_date <= CURRENT_DATE + 30
                  OR (
                      (COALESCE(
                          (SELECT value FROM public.mileage_log
                           WHERE vehicle_id = v.id
                           ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          l.km_start
                      ) - l.km_start)::FLOAT
                      / GREATEST(CURRENT_DATE - l.start_date, 1)
                      * (l.end_date - l.start_date) > l.km_allowed
                  )
              )
        ) OR EXISTS (
            SELECT 1 FROM public.contracts_insurance i
            WHERE i.vehicle_id = v.id
              AND i.end_date >= CURRENT_DATE
              AND (
                  i.end_date <= CURRENT_DATE + 30
                  OR (
                      (COALESCE(
                          (SELECT value FROM public.mileage_log
                           WHERE vehicle_id = v.id
                           ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          i.km_start
                      ) - i.km_start)::FLOAT
                      / GREATEST(CURRENT_DATE - i.start_date, 1)
                      * (i.end_date - i.start_date) > i.km_annual_limit
                  )
              )
        ) THEN 'warning'

        -- ok : au moins un contrat en cours, non dépassé
        WHEN EXISTS (
            SELECT 1 FROM public.contracts_loa l
            WHERE l.vehicle_id = v.id AND l.end_date >= CURRENT_DATE
        ) OR EXISTS (
            SELECT 1 FROM public.contracts_insurance i
            WHERE i.vehicle_id = v.id AND i.end_date >= CURRENT_DATE
        ) THEN 'ok'

        ELSE NULL
    END AS status
FROM public.vehicles v;
