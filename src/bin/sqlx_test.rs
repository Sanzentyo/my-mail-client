use anyhow::Result;
use sqlx;
use std::path::Path;

#[derive(Debug, sqlx::FromRow)]
struct DataBaseMsg {
    from_: String,
    to_: String,
    content: String,
    uuid: i64,
    connected_msg_uuid: i64,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = "./test.db";
    
    // データベースファイルが存在しない場合、空のファイルを作成
    if !Path::new(db_path).exists() {
        std::fs::File::create(db_path)?;
    }

    // データベース接続
    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;

    // テーブル作成
    sqlx::query("CREATE TABLE IF NOT EXISTS msg (
        from_ TEXT,
        to_ TEXT,
        content TEXT,
        uuid INTEGER PRIMARY KEY AUTOINCREMENT,
        connected_msg_uuid INTEGER,
        timestamp TIMESTAMP
    )").execute(&pool).await?;

    // 以下は元のコードと同じ
    sqlx::query("INSERT INTO msg (from_, to_, content, connected_msg_uuid) VALUES (?, ?, ?, ?)")
        .bind("田中")
        .bind("鈴木")
        .bind("お疲れ様です。先日の件について確認したいのですが、お時間ありますでしょうか？")
        .bind(-1)
        .execute(&pool).await?;

    let messages: Vec<DataBaseMsg> = sqlx::query_as::<_, DataBaseMsg>("SELECT * FROM msg").fetch_all(&pool).await?;
    println!("{:?}", messages);

    // プールを明示的にクローズ
    pool.close().await;

    Ok(())
}