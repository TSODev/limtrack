-- Migration : Rôle Administrateur de Flotte
-- À exécuter sur NeonDB dans l'ordre

-- 1. Table des entreprises
CREATE TABLE public.companies (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    siret      TEXT,
    created_by UUID NOT NULL REFERENCES public.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Table des organisations (2 niveaux max : département → service)
--    parent_org_id = NULL → niveau 1 (département)
--    parent_org_id IS NOT NULL → niveau 2 (service)
CREATE TABLE public.organizations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id    UUID NOT NULL REFERENCES public.companies(id) ON DELETE CASCADE,
    parent_org_id UUID REFERENCES public.organizations(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Table des membres d'entreprise
CREATE TABLE public.company_members (
    user_id    UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES public.companies(id) ON DELETE CASCADE,
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, company_id)
);

-- 4. Table des rôles de flotte
--    org_id IS NULL  → rôle global (toute l'entreprise)
--    org_id NOT NULL → rôle local (une organisation)
CREATE TABLE public.fleet_roles (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES public.companies(id) ON DELETE CASCADE,
    org_id     UUID REFERENCES public.organizations(id) ON DELETE CASCADE,
    role       TEXT NOT NULL CHECK (role IN ('fleet_admin', 'fleet_viewer')),
    granted_by UUID NOT NULL REFERENCES public.users(id),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index partiels (NULL != NULL dans UNIQUE standard PostgreSQL)
CREATE UNIQUE INDEX fleet_roles_global_unique
    ON public.fleet_roles (user_id, company_id)
    WHERE org_id IS NULL;

CREATE UNIQUE INDEX fleet_roles_org_unique
    ON public.fleet_roles (user_id, company_id, org_id)
    WHERE org_id IS NOT NULL;

-- 5. Rattachement des véhicules à une entreprise / organisation
ALTER TABLE public.vehicles
    ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES public.companies(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS org_id     UUID REFERENCES public.organizations(id) ON DELETE SET NULL;
