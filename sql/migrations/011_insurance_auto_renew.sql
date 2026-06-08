-- Migration 011 : renouvellement automatique des contrats assurance
ALTER TABLE public.contracts_insurance
    ADD COLUMN auto_renew BOOLEAN NOT NULL DEFAULT FALSE;
