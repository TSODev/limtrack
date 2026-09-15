-- Migration 015 : carnet d'entretien (types récurrents + journal d'interventions)
CREATE TABLE public.maintenance_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    interval_km INT NULL CHECK (interval_km IS NULL OR interval_km > 0),
    interval_months INT NULL CHECK (interval_months IS NULL OR interval_months > 0),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (interval_km IS NOT NULL OR interval_months IS NOT NULL)
);
CREATE INDEX idx_maintenance_types_vehicle ON public.maintenance_types(vehicle_id);

CREATE TABLE public.maintenance_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    maintenance_type_id UUID NULL REFERENCES public.maintenance_types(id) ON DELETE SET NULL,
    label TEXT NOT NULL,
    performed_at DATE NOT NULL,
    km_at_service INT NOT NULL CHECK (km_at_service >= 0),
    cost FLOAT NULL,
    provider TEXT NULL,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_maintenance_entries_vehicle ON public.maintenance_entries(vehicle_id);
CREATE INDEX idx_maintenance_entries_type ON public.maintenance_entries(maintenance_type_id);
