use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::s3_profile::{S3Profile, S3ProfileInput};
use anyhow::Result;

#[async_trait]
pub trait S3ProfileRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<S3Profile>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<S3Profile>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<S3Profile>>;
    async fn create(&self, input: &S3ProfileInput) -> Result<S3Profile>;
    async fn update(&self, id: &Uuid, input: &S3ProfileInput) -> Result<S3Profile>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
}
