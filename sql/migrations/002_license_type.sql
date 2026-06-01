-- Migration 002 — Type de licence (personal | fleet)
-- À appliquer sur NeonDB via psql ou l'interface Neon

ALTER TABLE public.license_tokens
    ADD COLUMN IF NOT EXISTS license_type TEXT NOT NULL DEFAULT 'personal';
