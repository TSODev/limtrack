
-- On supprime l'ancienne table si elle a été partiellement créée
DROP TABLE IF EXISTS public.vehicle_access;
DROP TABLE IF EXISTS public.vehicles;

-- 1. Table des véhicules (un utilisateur possède plusieurs véhicules)
-- Modification de la table vehicles pour pointer vers "user"
-- On recrée avec le type TEXT pour owner_id
CREATE TABLE public.vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- owner_id DOIT être TEXT pour correspondre à neon_auth."user".id
    owner_id UUID NOT NULL, 
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    plate_number TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Contrainte de clé étrangère explicite
    CONSTRAINT vehicles_owner_id_fkey 
        FOREIGN KEY (owner_id) 
        REFERENCES neon_auth."user"(id) 
        ON DELETE CASCADE
);

-- 2. Table des contrats (un véhicule possède plusieurs contrats)
CREATE TABLE public.contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    contract_type TEXT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Table des kilométrages (historique par véhicule)
CREATE TABLE public.mileage_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    value INTEGER NOT NULL,
    recorded_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Idem pour la table de partage
-- On recrée la table de partage sur le même principe
CREATE TABLE public.vehicle_access (
    vehicle_id UUID NOT NULL REFERENCES public.vehicles(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES neon_auth."user"(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'viewer',
    PRIMARY KEY (vehicle_id, user_id)
);
