

use sqlx::{PgPool, Result};
use uuid::Uuid;
use super::model::Task;

pub async fn create_task(pool: &PgPool, user_id: Uuid, title: &str) -> Result<Task> {
    let rec = sqlx::query_as!(
        Task,
        r#"
        INSERT INTO tasks (id, user_id, title)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, done, created_at
        "#,
        Uuid::new_v4(),
        user_id,
        title,
    )
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn list_tasks(pool: &PgPool, user_id: Uuid) -> Result<Vec<Task>> {
    let rec = sqlx::query_as!(
        Task,
        r#"
        SELECT id, user_id, title, done, created_at
        FROM tasks
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rec)
}

pub async fn update_task(pool: &PgPool, user_id: Uuid, id: Uuid, title: Option<String>, done: Option<bool>) -> Result<Task> {
    let rec = sqlx::query_as!(
        Task,
        r#"
        UPDATE tasks
        SET
            title = COALESCE($3, title),
            done = COALESCE($4, done)
        WHERE id = $2 AND user_id = $1
        RETURNING id, user_id, title, done, created_at
        "#,
        user_id, id, title, done
    ).fetch_one(pool).await?;

    Ok(rec)
}

pub async fn delete_task(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM tasks
        WHERE id = $1 AND user_id = $2
        "#,
        id, user_id
    ).execute(pool).await?;

    Ok(())
}