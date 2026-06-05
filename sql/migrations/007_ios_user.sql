-- Marque les comptes activés via l'App Store iOS (version Personal).
-- Ces utilisateurs n'ont pas accès aux fonctionnalités de gestion de flotte.
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS is_ios BOOLEAN NOT NULL DEFAULT FALSE;
