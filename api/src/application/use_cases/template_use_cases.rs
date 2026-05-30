use std::sync::Arc;
use anyhow::{Result, anyhow};
use chrono::Utc;
use uuid::Uuid;

use crate::domain::server::template::{TemplateRepository, Template};
use crate::application::dto::template_dtos::*;

// ── Create Template Use Case ─────────────────────────────────────────

pub struct CreateTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> CreateTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, req: CreateTemplateRequest) -> Result<Template> {
        let now = Utc::now().naive_utc();
        let template = Template {
            id: Uuid::new_v4(),
            game_type: req.game_type,
            category: req.category,
            display_name: req.display_name,
            description: req.description,
            config: req.config,
            visibility: req.visibility.unwrap_or_else(|| "private".to_string()),
            user_id: Some(user_id),
            is_builtin: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        self.repository.create_template(&template).await?;
        Ok(template)
    }
}

// ── List Templates Use Case ──────────────────────────────────────────

pub struct ListTemplatesUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> ListTemplatesUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, game_type: Option<String>) -> Result<Vec<Template>> {
        // Show public templates + user's own templates
        let mut templates = self.repository.list_public_templates().await?;
        let user_templates = self.repository.list_templates_by_user(user_id).await?;
        templates.extend(user_templates);

        // Deduplicate by id
        templates.sort_by(|a, b| a.id.cmp(&b.id));
        templates.dedup_by(|a, b| a.id == b.id);

        if let Some(gt) = game_type {
            templates.retain(|t| t.game_type == gt);
        }

        Ok(templates)
    }
}

// ── Get Template Use Case ────────────────────────────────────────────

pub struct GetTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> GetTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Template> {
        self.repository
            .get_template_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Template not found: {}", id))
    }
}

// ── Update Template Use Case ─────────────────────────────────────────

pub struct UpdateTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> UpdateTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, id: Uuid, req: UpdateTemplateRequest) -> Result<Template> {
        // Fetch existing template
        let existing = self.repository
            .get_template_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Template not found: {}", id))?;

        // Ownership check
        if existing.user_id != Some(user_id) {
            return Err(anyhow!("Forbidden"));
        }

        // Built-in templates cannot be updated by non-admin users
        if existing.is_builtin {
            return Err(anyhow!("Cannot update built-in template"));
        }

        // Apply partial updates
        let updated = Template {
            display_name: req.display_name.unwrap_or(existing.display_name),
            description: req.description.or(existing.description),
            config: req.config.unwrap_or(existing.config),
            visibility: req.visibility.unwrap_or(existing.visibility),
            category: req.category.unwrap_or(existing.category),
            ..existing
        };

        self.repository.update_template(&updated).await
    }
}

// ── Delete Template Use Case ─────────────────────────────────────────

pub struct DeleteTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> DeleteTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, id: Uuid) -> Result<()> {
        let template = self.repository
            .get_template_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Template not found: {}", id))?;

        // Ownership check
        if template.user_id != Some(user_id) {
            return Err(anyhow!("Forbidden"));
        }

        // Built-in templates cannot be deleted
        if template.is_builtin {
            return Err(anyhow!("Cannot delete built-in template"));
        }

        self.repository.delete_template(id).await?;
        Ok(())
    }
}

// ── Apply Template Use Case ──────────────────────────────────────────

/// Utility use case for deep-cloning a template's config to use during server creation.
pub struct ApplyTemplateUseCase<R: TemplateRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: TemplateRepository + ?Sized> ApplyTemplateUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Fetch a template by ID and return a deep clone of its config.
    pub async fn execute(&self, template_id: Uuid) -> Result<serde_json::Value> {
        let template = self.repository
            .get_template_by_id(template_id)
            .await?
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?;

        Ok(template.config.clone())
    }
}
