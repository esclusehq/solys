use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use anyhow::{Context, Result};

use crate::domain::entities::s3_profile::{S3Profile, S3ProfileInput};
use crate::domain::repositories::s3_profile_repository::S3ProfileRepository;

pub struct PostgresS3ProfileRepository {
    pool: PgPool,
}

impl PostgresS3ProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl S3ProfileRepository for PostgresS3ProfileRepository {
    async fn list_all(&self) -> Result<Vec<S3Profile>> {
        let rows = sqlx::query(
            "SELECT id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at FROM s3_profiles ORDER BY name ASC"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list S3 profiles")?;

        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(S3Profile {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                endpoint: row.try_get("endpoint")?,
                region: row.try_get("region")?,
                bucket: row.try_get("bucket")?,
                access_key: row.try_get("access_key")?,
                secret_key: row.try_get("secret_key")?,
                is_default: row.try_get("is_default")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(profiles)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<S3Profile>> {
        let row = sqlx::query(
            "SELECT id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at FROM s3_profiles WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find S3 profile by id")?;

        Ok(row.map(|r| S3Profile {
            id: r.try_get("id").unwrap(),
            name: r.try_get("name").unwrap(),
            endpoint: r.try_get("endpoint").unwrap(),
            region: r.try_get("region").unwrap(),
            bucket: r.try_get("bucket").unwrap(),
            access_key: r.try_get("access_key").unwrap(),
            secret_key: r.try_get("secret_key").unwrap(),
            is_default: r.try_get("is_default").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
            updated_at: r.try_get("updated_at").unwrap(),
        }))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<S3Profile>> {
        let row = sqlx::query(
            "SELECT id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at FROM s3_profiles WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find S3 profile by name")?;

        Ok(row.map(|r| S3Profile {
            id: r.try_get("id").unwrap(),
            name: r.try_get("name").unwrap(),
            endpoint: r.try_get("endpoint").unwrap(),
            region: r.try_get("region").unwrap(),
            bucket: r.try_get("bucket").unwrap(),
            access_key: r.try_get("access_key").unwrap(),
            secret_key: r.try_get("secret_key").unwrap(),
            is_default: r.try_get("is_default").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
            updated_at: r.try_get("updated_at").unwrap(),
        }))
    }

    async fn create(&self, input: &S3ProfileInput) -> Result<S3Profile> {
        let row = sqlx::query(
            r#"
            INSERT INTO s3_profiles (name, endpoint, region, bucket, access_key, secret_key, is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at
            "#
        )
        .bind(&input.name)
        .bind(&input.endpoint)
        .bind(input.region.as_deref().unwrap_or(""))
        .bind(&input.bucket)
        .bind(&input.access_key)
        .bind(input.secret_key.as_deref().unwrap_or(""))
        .bind(input.is_default.unwrap_or(false))
        .fetch_one(&self.pool)
        .await
        .context("Failed to create S3 profile")?;

        Ok(S3Profile {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            endpoint: row.try_get("endpoint")?,
            region: row.try_get("region")?,
            bucket: row.try_get("bucket")?,
            access_key: row.try_get("access_key")?,
            secret_key: row.try_get("secret_key")?,
            is_default: row.try_get("is_default")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    async fn update(&self, id: &Uuid, input: &S3ProfileInput) -> Result<S3Profile> {
        // If secret_key is None or empty, fetch existing and keep it
        let secret_key = if input.secret_key.as_deref().unwrap_or("").is_empty() {
            let existing = self.find_by_id(id).await?;
            existing.map(|p| p.secret_key).unwrap_or_default()
        } else {
            input.secret_key.clone().unwrap_or_default()
        };

        let row = sqlx::query(
            r#"
            UPDATE s3_profiles SET
                name = $1, endpoint = $2, region = $3, bucket = $4,
                access_key = $5, secret_key = $6, is_default = $7,
                updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, endpoint, region, bucket, access_key, secret_key, is_default, created_at, updated_at
            "#
        )
        .bind(&input.name)
        .bind(&input.endpoint)
        .bind(input.region.as_deref().unwrap_or(""))
        .bind(&input.bucket)
        .bind(&input.access_key)
        .bind(&secret_key)
        .bind(input.is_default.unwrap_or(false))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update S3 profile")?;

        Ok(S3Profile {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            endpoint: row.try_get("endpoint")?,
            region: row.try_get("region")?,
            bucket: row.try_get("bucket")?,
            access_key: row.try_get("access_key")?,
            secret_key: row.try_get("secret_key")?,
            is_default: row.try_get("is_default")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM s3_profiles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete S3 profile")?;
        Ok(())
    }
}
