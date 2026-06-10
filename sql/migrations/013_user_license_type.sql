-- Migration 013 : ajout de license_type sur users
-- Centralise le type de licence (personal/fleet) directement sur l'utilisateur
-- pour permettre l'édition admin et simplifier get_license.

ALTER TABLE public.users ADD COLUMN license_type TEXT NOT NULL DEFAULT 'personal';

-- Backfill depuis le dernier jeton utilisé par chaque utilisateur
UPDATE public.users u
SET license_type = lt.license_type
FROM (
    SELECT DISTINCT ON (used_by) used_by, license_type
    FROM public.license_tokens
    WHERE used_by IS NOT NULL AND used_at IS NOT NULL
    ORDER BY used_by, used_at DESC
) lt
WHERE u.id = lt.used_by;
