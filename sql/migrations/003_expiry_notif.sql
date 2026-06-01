-- Migration 003 — Suivi des notifications d'expiration de licence
-- À appliquer sur NeonDB via psql ou l'interface Neon

ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS expiry_notif_sent_at TIMESTAMPTZ;
