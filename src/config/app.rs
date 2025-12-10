use argon_core::config::ConfigBuilder;
use argon_macros::Config;

#[derive(Clone, Config)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
}

impl ConfigBuilder for AppConfig {
    fn build() -> anyhow::Result<Self> {
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
}
