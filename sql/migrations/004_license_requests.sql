-- Table de suivi des demandes de licence gratuite
-- Contrainte UNIQUE sur email : 1 jeton par adresse

CREATE TABLE public.license_requests (
    id           SERIAL PRIMARY KEY,
    email        TEXT NOT NULL UNIQUE,
    token_hash   TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
