-- Migration 020 : marquer un voyage planifié comme réalisé (clôture manuelle)
-- Purement informatif — le calcul de projection (usage-forecast) exclut déjà
-- automatiquement toute occurrence passée (from = today + 1 jour dans build_forecast),
-- qu'elle soit marquée réalisée ou non. Ce champ sert uniquement à distinguer dans
-- l'historique les voyages effectivement réalisés des voyages passés sans suite.
ALTER TABLE public.planned_trips ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE;
