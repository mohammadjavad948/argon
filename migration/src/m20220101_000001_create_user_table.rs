use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("user")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("name").not_null())
                    .col(string("username").unique_key().not_null())
                    .col(string("password").not_null())
                    .col(timestamp("created_at").default(Expr::current_timestamp()).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("user").to_owned())
            .await
    }
}
