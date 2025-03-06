use sqlx::SqlitePool;
use std::{str::FromStr, sync::Arc};

use sqlx::Row;

#[derive(Clone)]
pub struct DB {
    /// wrap in Arc to make it thread safe and zero cost to clone
    /// Sqlx will handle thread safety internally
    conn: Arc<SqlitePool>,
}

impl DB {
    /// open new connection to the database in memory
    pub async fn new() -> anyhow::Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")?;
        let conn = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("src/db/migrations").run(&conn).await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    /// delete listener data
    pub async fn delete_listener_data(&self, listener_id: usize) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM ListenerFrame WHERE Listener = ?")
            .bind(listener_id as i64)
            .execute(self.conn.as_ref())
            .await?;
        Ok(())
    }

    /// insert new listener frame data
    pub async fn insert_listener_frame_data(
        &self,
        listener_id: usize,
        frame_id: usize,
    ) -> anyhow::Result<()> {
        self.delete_listener_data(listener_id).await?;
        sqlx::query("INSERT INTO ListenerFrame (Listener, Frame) VALUES (?, ?)")
            .bind(listener_id as i64)
            .bind(frame_id as i64)
            .execute(&*self.conn)
            .await?;
        Ok(())
    }

    /// get the smallest frame id in ListenerFrame
    pub async fn get_smallest_frame_id(&self) -> anyhow::Result<Option<i64>> {
        let result = sqlx::query("SELECT Frame FROM ListenerFrame ORDER BY Frame ASC LIMIT 1")
            .fetch_optional(&*self.conn)
            .await?;
        Ok(result.map(|r| r.get(0)))
    }
}
