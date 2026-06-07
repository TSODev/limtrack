-- Migration 010 : table broadcasts (messages broadcast à tous les utilisateurs)
CREATE TABLE public.broadcasts (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    message     TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NULL,
    exclude_ios BOOLEAN     NOT NULL DEFAULT FALSE
);
