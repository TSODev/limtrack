-- Migration 019 : backfill vehicle_access manquant pour les propriétaires
-- create_vehicle n'a jamais inséré de ligne vehicle_access pour le owner (bug corrigé dans
-- vehicles_handler.rs::create_vehicle) — sans cette ligne, list_vehicles (JOIN, pas LEFT JOIN)
-- et tous les handlers de sous-ressources (mileage/contracts/trips/maintenance) rendent le
-- véhicule invisible/inaccessible à son propre propriétaire. Backfill idempotent : ne touche
-- que les véhicules pour lesquels le owner n'a pas déjà de ligne vehicle_access.
INSERT INTO public.vehicle_access (vehicle_id, user_id, role)
SELECT v.id, v.owner_id, 'owner'
FROM public.vehicles v
WHERE NOT EXISTS (
    SELECT 1 FROM public.vehicle_access va
    WHERE va.vehicle_id = v.id AND va.user_id = v.owner_id
);
