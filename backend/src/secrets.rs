pub async fn load_secrets() {
    dotenvy::dotenv().ok();
}
