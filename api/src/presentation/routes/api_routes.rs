use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put, delete, patch},
};

use crate::bootstrap::container::AppContainer;
use crate::presentation::handlers::auth_handlers::AuthHandlers;
use crate::presentation::handlers::billing_handlers::BillingHandlers;
use crate::presentation::handlers::user_handlers::UserHandlers;
use crate::presentation::handlers::agent_handlers::AgentHandlers;
use crate::presentation::handlers::job_handlers::JobHandlers;
use crate::presentation::handlers::webhook_handlers::WebhookHandlers;
use crate::presentation::handlers::subscription_handlers::SubscriptionHandlers;
use crate::presentation::handlers::server_handlers::ServerHandlers;
use crate::presentation::handlers::usage_handlers::UsageHandlers;

pub type ApiState = Arc<AppContainer>;

pub fn routes(container: Arc<AppContainer>) -> Router {
    let state = container.clone();
    
    tracing::info!("Building API routes...");
    
    let api_router = Router::new()
        // Auth & Users
        .nest("/api/v1/auth", AuthHandlers::router(state.clone()))
        .nest("/api/v1/billing", BillingHandlers::router(state.clone()))
        .nest("/api/v1/users", UserHandlers::router(state.clone()))
        
        // Server Management (nested under /api/v1/servers)
        .nest("/api/v1/servers", ServerHandlers::router(state.clone()))
        .route("/api/v1/servers/:server_id/backup-config", get(crate::presentation::handlers::backup_config_handlers::get_backup_config).put(crate::presentation::handlers::backup_config_handlers::update_backup_config))
        .route("/api/v1/servers/:server_id/tasks", get(crate::presentation::handlers::cron_task_handlers::list_tasks).post(crate::presentation::handlers::cron_task_handlers::create_task))
        .route("/api/v1/servers/:server_id/tasks/:task_id", patch(crate::presentation::handlers::cron_task_handlers::update_task).delete(crate::presentation::handlers::cron_task_handlers::delete_task))
        .route("/api/v1/servers/:server_id/tasks/:task_id/run", post(crate::presentation::handlers::cron_task_handlers::run_task))

        // Phase 67: Connectivity (per-server)
        .route("/api/v1/servers/:server_id/connectivity",
            get(crate::presentation::handlers::connectivity_handlers::get_status))
        .route("/api/v1/servers/:server_id/connectivity/probe",
            post(crate::presentation::handlers::connectivity_handlers::trigger_probe))
        .route("/api/v1/servers/:server_id/connectivity/audit",
            get(crate::presentation::handlers::connectivity_handlers::get_audit_log))

        // Phase 68: Relay (per-server nested under /api/v1/servers)
        .merge(crate::presentation::handlers::relay_handlers::router())
        
        // Agents & Nodes
        .nest("/api/v1/agents", AgentHandlers::router(state.clone()))
        .route("/api/v1/nodes", get(crate::presentation::handlers::node_handlers::list_nodes).post(crate::presentation::handlers::node_handlers::create_node))
        .route("/api/v1/nodes/:id", get(crate::presentation::handlers::node_handlers::get_node).put(crate::presentation::handlers::node_handlers::update_node).delete(crate::presentation::handlers::node_handlers::delete_node))
        .route("/api/v1/nodes/online", get(crate::presentation::handlers::node_handlers::list_online_nodes))
        .route("/api/v1/nodes/:id/status/:status", put(crate::presentation::handlers::node_handlers::update_node_status))
        .route("/api/v1/nodes/:id/metrics", get(crate::presentation::handlers::node_handlers::get_node_metrics))
        .route("/api/v1/nodes/:id/metrics/history/:limit", get(crate::presentation::handlers::node_handlers::get_node_metrics_history))
        .route("/api/v1/nodes/:id/resources", get(crate::presentation::handlers::node_handlers::get_node_resources))
        .route("/api/v1/nodes/:id/health", get(crate::presentation::handlers::node_handlers::get_node_health))
        .route("/api/v1/nodes/health/all", get(crate::presentation::handlers::node_handlers::list_nodes_health))
        .route("/api/v1/nodes/health/unhealthy", get(crate::presentation::handlers::node_handlers::list_unhealthy_nodes))
        .route("/api/v1/nodes/:id/keys", get(crate::presentation::handlers::node_handlers::list_api_keys))
        .route("/api/v1/nodes/:id/generate-key", post(crate::presentation::handlers::node_handlers::generate_node_key))
        .route("/api/v1/nodes/:node_id/keys/:key_id/revoke", put(crate::presentation::handlers::node_handlers::revoke_api_key))
        .route("/api/v1/nodes/:node_id/keys/:key_id", delete(crate::presentation::handlers::node_handlers::delete_api_key))
        .route("/api/v1/nodes/:id/tokens", get(crate::presentation::handlers::node_registration_token_handlers::list_registration_tokens).post(crate::presentation::handlers::node_registration_token_handlers::generate_registration_token))
        .route("/api/v1/nodes/:id/tokens/:token_id", delete(crate::presentation::handlers::node_registration_token_handlers::revoke_registration_token))
        .route("/api/v1/nodes/register", post(crate::presentation::handlers::node_registration_token_handlers::register_with_token))
        .route("/api/v1/nodes/:id/commands", post(crate::presentation::handlers::node_handlers::poll_node_commands))
        .route("/api/v1/nodes/:id/commands/result", post(crate::presentation::handlers::node_handlers::report_command_result))
        // Internal: Worker→Agent command dispatch (Phase 59)
        .route("/api/v1/nodes/:id/dispatch", post(crate::presentation::handlers::node_handlers::dispatch_node_command))
        .route("/api/ws/node", get(crate::presentation::handlers::node_ws_handler::ws_node_handler))
        
        // Jobs & Webhooks
        .nest("/api/v1/jobs", JobHandlers::router(state.clone()))
        .nest("/api/v1/webhooks", WebhookHandlers::router(state.clone()))
        .nest("/api/v1/subscriptions", SubscriptionHandlers::router(state.clone()))
        
        // Plugins (global)
        .route("/api/v1/plugins/search", get(crate::presentation::handlers::plugin_handlers::search_plugins))
        .route("/api/v1/plugins/:project_id/versions", get(crate::presentation::handlers::plugin_handlers::get_plugin_versions))
        
        // Alert Rules (global)
        .route("/api/v1/alert-rules", post(crate::presentation::handlers::alert_handlers::create_rule).get(crate::presentation::handlers::alert_handlers::list_rules))
        .route("/api/v1/alert-rules/:id", get(crate::presentation::handlers::alert_handlers::get_rule).put(crate::presentation::handlers::alert_handlers::update_rule).delete(crate::presentation::handlers::alert_handlers::delete_rule))
        .route("/api/v1/alert-history", get(crate::presentation::handlers::alert_handlers::list_history))
        
        // Settings
        .route("/api/v1/settings/s3", get(crate::presentation::handlers::settings_handlers::get_s3_config).put(crate::presentation::handlers::settings_handlers::save_s3_config))
        .route("/api/v1/settings/s3/profiles", get(crate::presentation::handlers::settings_handlers::list_s3_profiles).post(crate::presentation::handlers::settings_handlers::create_s3_profile))
        .route("/api/v1/settings/s3/profiles/:id", get(crate::presentation::handlers::settings_handlers::get_s3_profile).put(crate::presentation::handlers::settings_handlers::update_s3_profile).delete(crate::presentation::handlers::settings_handlers::delete_s3_profile))
        .route("/api/v1/settings/cloudflare", get(crate::presentation::handlers::settings_handlers::get_cloudflare_config).put(crate::presentation::handlers::settings_handlers::save_cloudflare_config))
        .route("/api/v1/settings/cloudflare/test", post(crate::presentation::handlers::settings_handlers::test_cloudflare_config))
        .route("/api/v1/settings/restart-defaults", get(crate::presentation::handlers::settings_handlers::get_restart_defaults).put(crate::presentation::handlers::settings_handlers::save_restart_defaults))
        // Modrinth API key (admin only)
        .route("/api/v1/settings/modrinth-api-key",
            get(crate::presentation::handlers::settings_handlers::get_modrinth_api_key)
            .put(crate::presentation::handlers::settings_handlers::save_modrinth_api_key))
        // CurseForge API key (admin only)
        .route("/api/v1/settings/curseforge-api-key",
            get(crate::presentation::handlers::settings_handlers::get_curseforge_api_key)
            .put(crate::presentation::handlers::settings_handlers::save_curseforge_api_key))
        
        // Deploy (global)
        .route("/api/v1/deploy/projects", get(crate::presentation::handlers::deployment_handlers::get_modrinth_projects))
        .route("/api/v1/deploy/servers", get(crate::presentation::handlers::deployment_handlers::get_production_servers))
        
        // Runtimes
        .route("/api/v1/runtimes", get(crate::presentation::handlers::runtime_handlers::get_available_runtimes))
        
        // Templates
        .route("/api/v1/templates", get(crate::presentation::handlers::template_handlers::list_templates)
            .post(crate::presentation::handlers::template_handlers::create_template))
        .route("/api/v1/templates/:id", get(crate::presentation::handlers::template_handlers::get_template)
            .put(crate::presentation::handlers::template_handlers::update_template)
            .delete(crate::presentation::handlers::template_handlers::delete_template))
        .route("/api/v1/templates/:id/create-server", post(crate::presentation::handlers::template_handlers::apply_template_to_server))
        
        // Plugin Templates
        .route("/api/v1/plugin-templates", get(crate::presentation::handlers::plugin_template_handlers::list_plugin_templates))
        
        // Modpack Templates
        .route("/api/v1/modpack-templates", get(crate::presentation::handlers::modpack_template_handlers::list_modpack_templates))
        
        // WebSocket
        .route("/ws", get(crate::presentation::handlers::ws_handler::ws_handler))
        .route("/ws/docker-logs", get(crate::presentation::handlers::docker_log_handler::docker_logs_ws))
        .route("/ws/terminal/:server_id", get(crate::presentation::handlers::terminal_ws_handler::ws_terminal))
        
        // Health
        .route("/health", get(health_check));
    
    tracing::info!("API routes built - testing /api/v1/jobs");
    
    api_router.with_state(state)
}

async fn health_check() -> &'static str {
    r#"{"status":"ok"}"#
}
