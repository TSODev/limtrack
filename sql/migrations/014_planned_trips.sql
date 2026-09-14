-- Migration 014 : planification de voyages futurs (ponctuels ou récurrents)
CREATE TABLE public.planned_trips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    estimated_km INT NOT NULL CHECK (estimated_km > 0),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    recurrence TEXT NOT NULL DEFAULT 'none',        -- 'none' | 'daily' | 'weekly' | 'monthly'
    recurrence_interval INT NOT NULL DEFAULT 1,     -- tous les N jours/semaines/mois
    days_of_week SMALLINT[] NULL,                   -- hebdo uniquement, 0=lundi..6=dimanche
    day_of_month SMALLINT NULL,                     -- mensuel uniquement, 1-31 (clampé en fin de mois)
    recurrence_end_date DATE NULL,                  -- NULL = expansion jusqu'à la fin du contrat actif
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_planned_trips_vehicle ON public.planned_trips(vehicle_id);
