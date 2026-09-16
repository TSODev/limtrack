-- Statut d'une entrée d'entretien : prevu (intention/rappel, pas encore fait),
-- devis (chiffrage reçu, toujours pas réalisé), realise (intervention effectuée).
-- Seul 'realise' compte pour le calcul de la prochaine échéance (compute_maintenance_status) —
-- 'prevu' et 'devis' sont traités de façon identique par le calcul, la distinction entre les
-- deux n'est qu'informative pour l'utilisateur.
ALTER TABLE public.maintenance_entries
    ADD COLUMN status TEXT NOT NULL DEFAULT 'prevu'
    CONSTRAINT maintenance_entries_status_check CHECK (status IN ('prevu', 'devis', 'realise'));

-- Backfill : toutes les entrées existantes représentent des interventions déjà réalisées
-- (créées avant l'existence de ce statut, donc forcément loggées après coup).
UPDATE public.maintenance_entries SET status = 'realise';
