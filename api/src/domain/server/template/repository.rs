use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::Template;

/// Repository trait for template operations.
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn list_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_templates_by_game(&self, game_type: &str) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_template(&self, game_type: &str, category: &str) -> Result<Option<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_template_by_id(&self, id: Uuid) -> Result<Option<Template>, Box<dyn std::error::Error + Send + Sync>>;

    // NEW CRUD methods
    async fn create_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_template(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn list_templates_by_user(&self, user_id: Uuid) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_public_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>>;
}

/// SQLx implementation of TemplateRepository.
pub struct SqlxTemplateRepository {
    pool: PgPool,
}

impl SqlxTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemplateRepository for SqlxTemplateRepository {
    /// List all templates from database, with fallback to hardcoded templates if empty.
    async fn list_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE is_active = true
            ORDER BY game_type, category
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        // If no templates in DB, return fallback templates
        if result.is_empty() {
            tracing::info!("No templates found in database, using fallback templates");
            return Ok(Template::fallback());
        }

        Ok(result)
    }

    /// List templates filtered by game type, with fallback to hardcoded templates if empty.
    async fn list_templates_by_game(&self, game_type: &str) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE game_type = $1 AND is_active = true
            ORDER BY category
            "#
        )
        .bind(game_type)
        .fetch_all(&self.pool)
        .await?;

        // If no templates in DB for this game type, return fallback
        if result.is_empty() {
            tracing::info!("No templates found for game_type={}, using fallback", game_type);
            return Ok(Template::fallback_by_game_type(game_type));
        }

        Ok(result)
    }

    /// Get a specific template by game type and category.
    async fn get_template(&self, game_type: &str, category: &str) -> Result<Option<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE game_type = $1 AND category = $2 AND is_active = true
            "#
        )
        .bind(game_type)
        .bind(category)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Get a template by its ID.
    async fn get_template_by_id(&self, id: Uuid) -> Result<Option<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE id = $1 AND is_active = true
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Create a new template in the database. Returns the created template.
    async fn create_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            INSERT INTO templates (id, game_type, category, display_name, description, config,
                                   visibility, user_id, is_builtin, is_active,
                                   created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, game_type, category, display_name, description, config,
                      visibility, user_id, is_builtin, is_active,
                      created_at, updated_at
            "#
        )
        .bind(template.id)
        .bind(&template.game_type)
        .bind(&template.category)
        .bind(&template.display_name)
        .bind(&template.description)
        .bind(&template.config)
        .bind(&template.visibility)
        .bind(template.user_id)
        .bind(template.is_builtin)
        .bind(template.is_active)
        .bind(template.created_at)
        .bind(template.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Update an existing template. Updates display_name, description, config, category, visibility.
    /// Does NOT update id, game_type, user_id, is_builtin, is_active, created_at.
    async fn update_template(&self, template: &Template) -> Result<Template, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            UPDATE templates
            SET display_name = $1, description = $2, config = $3, category = $4,
                visibility = $5, updated_at = NOW()
            WHERE id = $6
            RETURNING id, game_type, category, display_name, description, config,
                      visibility, user_id, is_builtin, is_active,
                      created_at, updated_at
            "#
        )
        .bind(&template.display_name)
        .bind(&template.description)
        .bind(&template.config)
        .bind(&template.category)
        .bind(&template.visibility)
        .bind(template.id)
        .fetch_optional(&self.pool)
        .await?;

        result.ok_or_else(|| {
            Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))
                as Box<dyn std::error::Error + Send + Sync>
        })
    }

    /// Delete a template by ID. This performs a hard DELETE (not soft-delete).
    /// The handler layer checks is_builtin before calling this.
    async fn delete_template(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            DELETE FROM templates
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List templates owned by a specific user.
    async fn list_templates_by_user(&self, user_id: Uuid) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        // Fallback is not applicable for user-specific queries
        Ok(result)
    }

    /// List public templates and built-in templates (visibility = 'public' OR is_builtin = true).
    /// Built-in templates appear first in the result.
    async fn list_public_templates(&self) -> Result<Vec<Template>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, game_type, category, display_name, description, config,
                   visibility, user_id, is_builtin, is_active,
                   created_at, updated_at
            FROM templates
            WHERE is_active = true AND (visibility = 'public' OR is_builtin = true)
            ORDER BY is_builtin DESC, game_type, category
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        // If no templates in DB, return fallback templates
        if result.is_empty() {
            tracing::info!("No public templates found in database, using fallback templates");
            return Ok(Template::fallback());
        }

        Ok(result)
    }
}
