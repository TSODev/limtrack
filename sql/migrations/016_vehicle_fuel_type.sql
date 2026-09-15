-- Migration 016 : motorisation du véhicule (filtre le catalogue générique d'entretien)
ALTER TABLE public.vehicles
    ADD COLUMN fuel_type TEXT NULL CHECK (fuel_type IS NULL OR fuel_type IN ('thermique', 'electrique', 'hybride'));
