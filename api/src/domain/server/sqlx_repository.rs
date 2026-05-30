use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::server::model::Server;
use crate::domain::server::repository::ServerRepository;

pub struct SqlxServerRepository {
    pool: PgPool,
}

impl SqlxServerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, server: &Server) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("[repo create] Inserting server: id={}, user_id={:?}, name={}, image={}, status={}", 
            server.id, server.user_id, server.name, server.image, server.status);
        
        let result = sqlx::query_as::<_, Server>(
            r#"
            INSERT INTO servers (
                id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, 
                config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port,
                config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at
            "#,
        )
        .bind(server.id)
        .bind(server.user_id)
        .bind(server.agent_id)
        .bind(server.job_id)
        .bind(&server.name)
        .bind(&server.image)
        .bind(&server.executor_type)
        .bind(&server.node_id)
        .bind(&server.status)
        .bind(&server.remote_id)
        .bind(&server.port)
        .bind(&server.config)
        .bind(&server.resources)
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(&server.endpoints)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_all(&self) -> Result<Vec<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let results = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    pub async fn update(&self, server: &Server) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            r#"
            UPDATE servers 
            SET agent_id = $2, job_id = $3, name = $4, image = $5, executor_type = $6, node_id = $7, status = $8, remote_id = $9, port = $10,
                config = $11, resources = $12, auto_wake = $13, sleep_timeout_minutes = $14, last_restart_at = $15, last_restart_reason = $16, health_check_timeout_seconds = $17, endpoints = $18, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(server.id)
        .bind(server.agent_id)
        .bind(server.job_id)
        .bind(&server.name)
        .bind(&server.image)
        .bind(&server.executor_type)
        .bind(&server.node_id)
        .bind(&server.status)
        .bind(&server.remote_id)
        .bind(&server.port)
        .bind(&server.config)
        .bind(&server.resources)
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(&server.endpoints)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query("UPDATE servers SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ServerRepository for SqlxServerRepository {
    async fn create(&self, server: &Server) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            r#"
            INSERT INTO servers (id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING *
            "#,
        )
        .bind(server.id)
        .bind(server.user_id)
        .bind(server.agent_id)
        .bind(server.job_id)
        .bind(&server.name)
        .bind(&server.image)
        .bind(&server.executor_type)
        .bind(&server.node_id)
        .bind(&server.status)
        .bind(&server.remote_id)
        .bind(&server.port)
        .bind(&server.config)
        .bind(&server.resources)
            .bind(server.auto_wake)
            .bind(server.sleep_timeout_minutes)
            .bind(server.last_restart_at)
            .bind(&server.last_restart_reason)
            .bind(server.health_check_timeout_seconds)
            .bind(&server.endpoints)
            .bind(server.created_at)
        .bind(server.updated_at)
        .bind(server.deleted_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let results = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE user_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_agent_id(&self, agent_id: Uuid) -> Result<Vec<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let results = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE agent_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_job_id(&self, job_id: Uuid) -> Result<Option<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE job_id = $1 AND deleted_at IS NULL"
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<Server>, Box<dyn std::error::Error + Send + Sync>> {
        let results = sqlx::query_as::<_, Server>(
            "SELECT id, user_id, agent_id, job_id, name, image, executor_type, node_id, status, remote_id, port, config, resources, auto_wake, sleep_timeout_minutes, last_restart_at, last_restart_reason, health_check_timeout_seconds, endpoints, created_at, updated_at, deleted_at FROM servers WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, server: &Server) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Server>(
            r#"
            UPDATE servers 
            SET agent_id = $2, job_id = $3, name = $4, image = $5, status = $6, remote_id = $7, 
                config = $8, resources = $9, auto_wake = $10, sleep_timeout_minutes = $11, last_restart_at = $12, last_restart_reason = $13, health_check_timeout_seconds = $14, endpoints = $15, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(server.id)
        .bind(server.agent_id)
        .bind(server.job_id)
        .bind(&server.name)
        .bind(&server.image)
        .bind(&server.status)
        .bind(&server.remote_id)
        .bind(&server.config)
        .bind(&server.resources)
        .bind(server.auto_wake)
        .bind(server.sleep_timeout_minutes)
        .bind(server.last_restart_at)
        .bind(&server.last_restart_reason)
        .bind(server.health_check_timeout_seconds)
        .bind(&server.endpoints)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query("UPDATE servers SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_scalar(
            "SELECT COUNT(*) FROM servers WHERE user_id = $1 AND deleted_at IS NULL"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
