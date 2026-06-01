-- Migration 001 — Système de licences par jetons
-- À appliquer sur NeonDB via psql ou l'interface Neon

-- Période d'essai et expiration d'accès sur les utilisateurs
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS trial_ends_at    TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '3 months',
    ADD COLUMN IF NOT EXISTS access_expires_at TIMESTAMPTZ;

-- Table des jetons de licence
CREATE TABLE IF NOT EXISTS public.license_tokens (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash    TEXT        NOT NULL UNIQUE,   -- SHA-256 du jeton en clair
    duration_days INT         NOT NULL,           -- 30, 90, 365...
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used_at       TIMESTAMPTZ,
    used_by       UUID        REFERENCES public.users(id) ON DELETE SET NULL
);
