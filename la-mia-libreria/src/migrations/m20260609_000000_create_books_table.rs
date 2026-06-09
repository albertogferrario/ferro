use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Books::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Books::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Books::Title).string().not_null())
                    .col(ColumnDef::new(Books::Author).string().null())
                    .col(ColumnDef::new(Books::Isbn).string().null())
                    .col(ColumnDef::new(Books::CoverUrl).string().null())
                    .col(ColumnDef::new(Books::Description).text().null())
                    .col(ColumnDef::new(Books::Year).integer().null())
                    // Catalog provenance: which source and the source's own id, so the
                    // same book can be re-found and de-duplicated on import.
                    .col(ColumnDef::new(Books::Source).string().not_null())
                    .col(ColumnDef::new(Books::SourceId).string().not_null())
                    // Public-domain books carry a real download URL we are allowed to fetch.
                    .col(
                        ColumnDef::new(Books::PublicDomain)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Books::DownloadUrl).string().null())
                    // Set once the file has been downloaded into local storage.
                    .col(ColumnDef::new(Books::LocalPath).string().null())
                    // Collection status: "wanted" | "owned" | "reading" | "read".
                    .col(
                        ColumnDef::new(Books::Status)
                            .string()
                            .not_null()
                            .default("wanted"),
                    )
                    .col(
                        ColumnDef::new(Books::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Books::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Prevent the same catalog entry from being added twice.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_books_source_source_id")
                    .table(Books::Table)
                    .col(Books::Source)
                    .col(Books::SourceId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Books::Table).to_owned())
            .await
    }
}

/// Table and column identifiers for books
#[derive(DeriveIden)]
enum Books {
    Table,
    Id,
    Title,
    Author,
    Isbn,
    CoverUrl,
    Description,
    Year,
    Source,
    SourceId,
    PublicDomain,
    DownloadUrl,
    LocalPath,
    Status,
    CreatedAt,
    UpdatedAt,
}
