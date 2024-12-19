use anyhow::Result;
use sqlx;

#[derive(Debug, sqlx::FromRow)]
struct DataBaseMsg {
    from_: String,
    to_: String,
    content: String,
    uuid: i64,
    connected_msg_uuid: i64,
    timstamp: i64,
}

#[tokio::main]
async fn main() -> Result<()>{
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite://./test.db").await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS msg (
        from_ TEXT,
        to_ TEXT,
        content TEXT,
        uuid INTEGER PRIMARY KEY AUTOINCREMENT,
        connected_msg_uuid INTEGER,
        timestamp TIMESTAMP
    )").execute(&pool).await?;

    sqlx::query("INSERT INTO msg (from_, to_, content, uuid, connected_msg_uuid, timstamp) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("Alice")
        .bind("Bob")
        .bind("Hello, Bob!")
        .bind(1)
        .bind(0)
        .bind(0)
        .execute(&pool).await?;



    let messages: Vec<DataBaseMsg> = sqlx::query_as::<_, DataBaseMsg>("SELECT * FROM msg").fetch_all(&pool).await?;

    Ok(())
}