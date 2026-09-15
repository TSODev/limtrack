-- Migration 018 : entretien multi-points (une "révision" peut couvrir plusieurs types)
-- Remplace la relation mono-type (maintenance_entries.maintenance_type_id) par une table
-- de jointure many-to-many : une entrée reste un événement unique (une facture, un jeu de
-- photos, un coût) mais peut être rattachée à plusieurs types (vidange + filtres + ...).
CREATE TABLE public.maintenance_entry_types (
    entry_id UUID NOT NULL REFERENCES public.maintenance_entries(id) ON DELETE CASCADE,
    maintenance_type_id UUID NOT NULL REFERENCES public.maintenance_types(id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, maintenance_type_id)
);
CREATE INDEX idx_maintenance_entry_types_type ON public.maintenance_entry_types(maintenance_type_id);

-- Backfill depuis l'ancienne colonne mono-type
INSERT INTO public.maintenance_entry_types (entry_id, maintenance_type_id)
SELECT id, maintenance_type_id FROM public.maintenance_entries WHERE maintenance_type_id IS NOT NULL;

ALTER TABLE public.maintenance_entries DROP COLUMN maintenance_type_id;
