-- Migration 008 : réinitialisation du mot de passe
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS password_reset_token TEXT,
    ADD COLUMN IF NOT EXISTS password_reset_expires_at TIMESTAMPTZ;
