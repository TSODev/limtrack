-- Prix du kilomètre supplémentaire en cas de dépassement (optionnel)
ALTER TABLE public.contracts_loa
    ADD COLUMN IF NOT EXISTS price_per_extra_km FLOAT;
