pub async fn init_env() {
    dotenvy::dotenv().expect("cannot load the .env file. is it there?");
}
