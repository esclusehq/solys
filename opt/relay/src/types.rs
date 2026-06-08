use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single server authorized by a relay token.
/// Phase 69: authorize() returns `Vec<ServerMapping>` for 1:N token→server mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMapping {
    pub server_id: Uuid,
    pub subdomain: Option<String>,
}
