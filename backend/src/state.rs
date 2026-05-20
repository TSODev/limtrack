// src/state.rs

use axum::extract::FromRef;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// Permet aux handlers qui font State<PgPool> de continuer à fonctionner
// sans modification, en extrayant le PgPool depuis AppState automatiquement.
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> PgPool {
        state.db.clone()
    }
}
