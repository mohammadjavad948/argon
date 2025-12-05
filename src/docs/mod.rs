use tokio::io::AsyncWriteExt;
use utoipa::OpenApi;

use crate::app::controller::TestControllerApi;
use crate::app::response::SimpleResponse;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = TestControllerApi)
    ),
    components(schemas(SimpleResponse)),
    info(description = "API Docs")
)]
pub struct MainApiDoc;

pub async fn generate_docs() -> anyhow::Result<()> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)     // create if not exists
        .truncate(true)   // truncates existing file → overwrites
        .open("api.json")
        .await?;

    let docs = MainApiDoc::openapi().to_pretty_json()?;

    file.write_all(docs.as_bytes()).await?;

    Ok(())
}
