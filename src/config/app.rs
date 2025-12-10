use tokio::sync::OnceCell;

const APP: OnceCell<AppConfig> = OnceCell::const_new();

#[derive(Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String
}

impl AppConfig {
    fn new() -> anyhow::Result<Self> {
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| {
                tracing::warn!("cannot read `SERVER_PORT` defaulting to `3000`");

                "3000".into()
            })
            .parse()
            .unwrap_or_else(|err| {
                tracing::error!("cannot parse `SERVER_PORT`. defaulting to 3000 {:?}", err);
                3000
            });

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|err| anyhow::anyhow!("cannot read `DATABASE_URL`: {:?}", err))?;

        Ok(AppConfig { port, database_url })
    }

    pub async fn get() -> AppConfig {
        APP
        .get_or_init(async || {
            AppConfig::new().unwrap()
        })
        .await
        .clone()
    }

    pub async fn port() -> u16 {
        APP
        .get_or_init(async || {
            AppConfig::new().unwrap()
        })
        .await
        .port
    }

    pub async fn database_url() -> String {
        APP
        .get_or_init(async || {
            AppConfig::new().unwrap()
        })
        .await
        .database_url
        .clone()
    }
}
