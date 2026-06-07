-- Migration 009 : archivage de véhicules
-- Un véhicule archivé disparaît de la liste principale mais conserve tout son historique.

ALTER TABLE public.vehicles ADD COLUMN archived_at TIMESTAMPTZ NULL;
