use anyhow::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct MsgDB {
    pub from_user: String,
    pub to_user: String,
    pub content: String,
    pub uuid: i64,
    pub connected_msg_uuid: i64,
    pub timestamp: i64,
}

const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS msg (
    from_user TEXT,
    to_user TEXT,
    content TEXT,
    uuid INTEGER PRIMARY KEY AUTOINCREMENT,
    connected_msg_uuid INTEGER,
    timestamp TIMESTAMP
)";

pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query(CREATE_TABLE).execute(pool).await?;
    Ok(())
}