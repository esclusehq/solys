use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::mpsc;

use crate::domain::{
    repositories::server_repository::ServerRepository,
    repositories::metrics_repository::MetricsRepository,
    factories::ExecutorFactory,
};
use crate::infrastructure::{
    repositories::postgres_server_repository::PostgresServerRepository,
    repositories::postgres_metrics_repository::PostgresMetricsRepository,
    repositories::postgres_alert_repository::PostgresAlertRepository,
    repositories::postgres_alert_state_repository::PostgresAlertStateRepository,
    repositories::postgres_alert_history_repository::PostgresAlertHistoryRepository,
    repositories::postgres_backup_config_repository::PostgresBackupConfigRepository,
    repositories::postgres_node_repository::PostgresNodeRepository,
    repositories::postgres_node_metrics_repository::PostgresNodeMetricsRepository,
    repositories::postgres_node_api_key_repository::PostgresNodeApiKeyRepository,
    repositories::postgres_node_registration_token_repository::PostgresNodeRegistrationTokenRepository,
    repositories::postgres_cron_task_repository::PostgresCronTaskRepository,
    node_client::NodeClient,
    events::event_bus::EventBus,
    factories::simple_executor_factory::SimpleExecutorFactory,
    solys_client::client::SolysClient,
    billing::BillingService,
    cache::redis::RedisPool,
};
use crate::domain::repositories::alert_repository::{AlertRepository, AlertStateRepository, AlertHistoryRepository};
use crate::application::use_cases::{
    create_server_use_case::CreateServerUseCase,
    list_servers_use_case::ListServersUseCase,
    get_server_use_case::GetServerUseCase,
    update_server_use_case::UpdateServerUseCase,
    delete_server_use_case::DeleteServerUseCase,
    start_server_use_case::StartServerUseCase,
    stop_server_use_case::StopServerUseCase,
    get_server_snapshot_use_case::GetServerSnapshotUseCase,
    evaluate_alerts_use_case::EvaluateAlertsUseCase,
    send_command_use_case::SendCommandUseCase,
    alert_use_cases::{
        CreateAlertRuleUseCase, ListAlertRulesUseCase, GetAlertRuleUseCase,
        UpdateAlertRuleUseCase, DeleteAlertRuleUseCase, ListAlertHistoryUseCase,
    },
    plugin_use_cases::{
        SearchPluginsUseCase, GetPluginVersionsUseCase, InstallPluginUseCase,
    },
    template_use_cases::{
        CreateTemplateUseCase, ListTemplatesUseCase, GetTemplateUseCase,
        UpdateTemplateUseCase, DeleteTemplateUseCase, ApplyTemplateUseCase,
    },
};
use crate::infrastructure::external_services::modrinth_client::ModrinthClient;
use crate::infrastructure::external_services::discord_client::DiscordClient;
use crate::infrastructure::repositories::postgres_backup_repository::PostgresBackupRepository;
use crate::infrastructure::repositories::postgres_settings_repository::PostgresSettingsRepository;
use crate::infrastructure::storage::local_client::LocalClient;
use crate::infrastructure::storage::s3_client::S3Client;
use crate::domain::repositories::backup_repository::BackupRepository;
use crate::domain::repositories::settings_repository::SettingsRepository;
use crate::domain::repositories::node_repository::NodeRepository;
use crate::domain::repositories::node_metrics_repository::NodeMetricsRepository;
use crate::domain::repositories::node_api_key_repository::NodeApiKeyRepository;
use crate::domain::repositories::node_registration_token_repository::NodeRegistrationTokenRepository;
use crate::domain::repositories::cron_task_repository::CronTaskRepository;
use crate::domain::repositories::backup_config_repository::BackupConfigRepository;
use crate::application::services::monitoring_service::{MonitoringService, CrashReportData};
use crate::presentation::ws::node_connection_manager::NodeConnectionManager;
use crate::application::services::webhook_service::WebhookService;
use crate::application::services::backup_service::BackupService;
use crate::application::services::backup_scheduler::BackupScheduler;
use crate::application::services::node_health_service::NodeHealthService;
use crate::domain::rbac::SqlxRbacRepository;
use crate::domain::server::template::{SqlxTemplateRepository, TemplateRepository};

/// AppContainer holds all shared dependencies for the application.
#[derive(Clone)]
pub struct AppContainer {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub app_url: String,
    pub billing_service: Arc<dyn BillingService>,
    pub billing_webhook_secret: Option<String>,
    pub solys_client: Arc<SolysClient>,
    pub create_server_use_case: Arc<CreateServerUseCase<dyn ServerRepository>>,
    pub list_servers_use_case: Arc<ListServersUseCase<dyn ServerRepository>>,
    pub get_server_use_case: Arc<GetServerUseCase<dyn ServerRepository>>,
    pub update_server_use_case: Arc<UpdateServerUseCase<dyn ServerRepository>>,
    pub delete_server_use_case: Arc<DeleteServerUseCase<dyn ServerRepository, dyn ExecutorFactory>>,
    pub start_server_use_case: Arc<StartServerUseCase<dyn ServerRepository, dyn ExecutorFactory>>,
    pub stop_server_use_case: Arc<StopServerUseCase<dyn ServerRepository, dyn ExecutorFactory>>,
    pub send_command_use_case: Arc<SendCommandUseCase<dyn ServerRepository, dyn ExecutorFactory>>,
    pub get_server_snapshot_use_case: Arc<GetServerSnapshotUseCase>,
    pub evaluate_alerts_use_case: Arc<EvaluateAlertsUseCase>,
    pub monitoring_service: Arc<MonitoringService<dyn ServerRepository, dyn MetricsRepository, dyn ExecutorFactory, dyn NodeRepository>>,
    pub event_bus: Arc<EventBus>,
    pub alert_repository: Arc<dyn AlertRepository>,
    // Alert use cases
    pub create_alert_rule_use_case: Arc<CreateAlertRuleUseCase>,
    pub list_alert_rules_use_case: Arc<ListAlertRulesUseCase>,
    pub get_alert_rule_use_case: Arc<GetAlertRuleUseCase>,
    pub update_alert_rule_use_case: Arc<UpdateAlertRuleUseCase>,
    pub delete_alert_rule_use_case: Arc<DeleteAlertRuleUseCase>,
    pub list_alert_history_use_case: Arc<ListAlertHistoryUseCase>,
    pub metrics_repository: Arc<dyn MetricsRepository>,
    // Plugin use cases
    pub search_plugins_use_case: Arc<SearchPluginsUseCase>,
    pub get_plugin_versions_use_case: Arc<GetPluginVersionsUseCase>,
    pub install_plugin_use_case: Arc<InstallPluginUseCase<dyn ServerRepository>>,
    // Webhook service
    pub webhook_service: Arc<WebhookService<dyn ServerRepository>>,
    // Backup service
    pub backup_service: Arc<BackupService<dyn ServerRepository, dyn BackupRepository>>,
    // Backup scheduler
    pub backup_scheduler: Arc<BackupScheduler<dyn ServerRepository, dyn BackupRepository>>,
    // Settings repository (for S3 config)
    pub settings_repository: Arc<dyn SettingsRepository>,
    // Server repository (for direct filesystem access in plugin handlers)
    pub server_repository: Arc<dyn ServerRepository>,
    // Node repository (for Node Agent management)
    pub node_repository: Arc<dyn NodeRepository>,
    // Node metrics repository (for node metrics storage)
    pub node_metrics_repository: Arc<dyn NodeMetricsRepository>,
    // Node API key repository
    pub node_api_key_repository: Arc<dyn NodeApiKeyRepository>,
    // Node registration token repository
    pub node_registration_token_repository: Arc<dyn NodeRegistrationTokenRepository>,
    // Node connection manager (for WebSocket connections)
    pub node_connection_manager: Arc<NodeConnectionManager>,
    // Node client (for sending commands to nodes)
    pub node_client: Arc<dyn NodeClient>,
    // Node health service
    pub node_health_service: Option<Arc<NodeHealthService>>,
    // RBAC
    pub rbac_repository: Arc<SqlxRbacRepository>,
    // Redis pool (for terminal history, caching)
    pub redis_pool: Option<Arc<RedisPool>>,
    // Discord client for server events
    pub discord_client: Arc<DiscordClient>,
    // Scheduled tasks (cron jobs)
    pub cron_task_repository: Arc<dyn CronTaskRepository>,
    pub backup_config_repository: Arc<dyn BackupConfigRepository>,
    // Template use cases
    pub create_template_use_case: Arc<CreateTemplateUseCase<dyn TemplateRepository>>,
    pub list_templates_use_case: Arc<ListTemplatesUseCase<dyn TemplateRepository>>,
    pub get_template_use_case: Arc<GetTemplateUseCase<dyn TemplateRepository>>,
    pub update_template_use_case: Arc<UpdateTemplateUseCase<dyn TemplateRepository>>,
    pub delete_template_use_case: Arc<DeleteTemplateUseCase<dyn TemplateRepository>>,
    pub apply_template_use_case: Arc<ApplyTemplateUseCase<dyn TemplateRepository>>,
    // Crash report channel (Phase 60) — agent WS handler sends, MonitoringService drains
    pub crash_report_tx: Option<mpsc::Sender<CrashReportData>>,
}

impl AppContainer {
    pub async fn new(pool: PgPool) -> Self {
        // 0. Load Config
        let config = crate::config::AppConfig::new().unwrap_or_else(|_| crate::config::AppConfig {
            database_url: String::new(),
            server_host: String::new(),
            server_port: 8080,
            environment: String::new(),
            rust_log: String::new(),
            redis_url: String::new(),
            redis_pool_size: 10,
            jwt_secret: "dev-secret-key-min-32-chars-long".to_string(),
            jwt_access_token_expiry_minutes: 10080,
            jwt_refresh_token_expiry_days: 7,
            bcrypt_cost: 12,
            api_key_length: 32,
            rate_limit_per_minute: 60,
            rate_limit_per_hour: 1000,
            worker_id: String::new(),
            worker_poll_interval_ms: 1000,
            stripe_secret_key: None,
            stripe_webhook_secret: None,
            stripe_publishable_key: None,
            lemon_squeezy_api_key: None,
            lemon_squeezy_webhook_secret: None,
            resend_api_key: None,
            email_from: None,
            solys_default_url: String::new(),
            app_url: "http://localhost:5173".to_string(),
        });

        let solys_client = Arc::new(SolysClient::new(
            config.solys_default_url.clone(),
            "fb22a1d153f110b6e51fe481d5ec9acd85ac0bfea51152f7df9145619718c8a7".to_string(),
        ));

        // 1. Initialize Infrastructure
        let server_repository = Arc::new(PostgresServerRepository::new(pool.clone()));
        let metrics_repository = Arc::new(PostgresMetricsRepository::new(pool.clone()));
        let alert_repository_concrete = Arc::new(PostgresAlertRepository::new(pool.clone()));
        let alert_state_repo_concrete = Arc::new(PostgresAlertStateRepository::new(pool.clone()));
        let alert_history_repo_concrete = Arc::new(PostgresAlertHistoryRepository::new(pool.clone()));
        
        let node_repository_concrete = Arc::new(PostgresNodeRepository::new(pool.clone()));
        let node_metrics_repository_concrete = Arc::new(PostgresNodeMetricsRepository::new(pool.clone()));
        let node_api_key_repository_concrete = Arc::new(PostgresNodeApiKeyRepository::new(pool.clone()));
        let node_registration_token_repository_concrete = Arc::new(PostgresNodeRegistrationTokenRepository::new(pool.clone()));
        let node_connection_manager = Arc::new(NodeConnectionManager::new());
        
        // Create executor factory first (which creates its own node client internally)
        let executor_factory = Arc::new(SimpleExecutorFactory::with_node_client(
            node_repository_concrete.clone(),
            node_connection_manager.clone(),
        ));
        
        // Get the node client from executor factory
        let node_client = executor_factory.node_client()
            .expect("Executor factory should provide a node client");
        let event_bus = Arc::new(EventBus::new());

        // Initialize Redis pool (for terminal history and caching)
        let redis_pool = match crate::infrastructure::cache::redis::new_redis_pool(&config.redis_url).await {
            Ok(pool) => Some(Arc::new(pool)),
            Err(e) => {
                tracing::warn!("Failed to connect to Redis: {}. Terminal history disabled.", e);
                None
            }
        };

        // 2. Cast to Traits
        let repo: Arc<dyn ServerRepository> = server_repository;
        let metrics: Arc<dyn MetricsRepository> = metrics_repository;
        let alert_repo: Arc<dyn AlertRepository> = alert_repository_concrete;
        let alert_state_repo: Arc<dyn AlertStateRepository> = alert_state_repo_concrete;
        let alert_history_repo: Arc<dyn AlertHistoryRepository> = alert_history_repo_concrete;
        let factory: Arc<dyn ExecutorFactory> = executor_factory;
        let node_repo: Arc<dyn NodeRepository> = node_repository_concrete;
        let node_metrics_repo: Arc<dyn NodeMetricsRepository> = node_metrics_repository_concrete;
        let node_api_key_repo: Arc<dyn NodeApiKeyRepository> = node_api_key_repository_concrete;
        let node_registration_token_repo: Arc<dyn NodeRegistrationTokenRepository> = node_registration_token_repository_concrete;

        // Cron task repository
        let cron_task_repository_concrete = Arc::new(PostgresCronTaskRepository::new(pool.clone()));
        let cron_task_repository: Arc<dyn CronTaskRepository> = cron_task_repository_concrete;

        // Node health service
        let node_health_service = Arc::new(NodeHealthService::new(
            node_repo.clone(),
            node_metrics_repo.clone(),
            node_connection_manager.clone(),
        ));

        // RBAC repository
        let rbac_repository = Arc::new(SqlxRbacRepository::new(pool.clone()));

        // 3. Initialize Use Cases
        let create_server_use_case = Arc::new(CreateServerUseCase::new(repo.clone()));
        let list_servers_use_case = Arc::new(ListServersUseCase::new(repo.clone()));
        let get_server_use_case = Arc::new(GetServerUseCase::new(repo.clone()));
        let update_server_use_case = Arc::new(UpdateServerUseCase::new(repo.clone()));
        let delete_server_use_case = Arc::new(DeleteServerUseCase::new(repo.clone(), factory.clone()));
        let start_server_use_case = Arc::new(StartServerUseCase::new(repo.clone(), factory.clone(), event_bus.clone()));
        let stop_server_use_case = Arc::new(StopServerUseCase::new(repo.clone(), factory.clone(), event_bus.clone()));
        let send_command_use_case = Arc::new(SendCommandUseCase::new(repo.clone(), factory.clone()));
        let get_server_snapshot_use_case = Arc::new(GetServerSnapshotUseCase::new(repo.clone(), metrics.clone()));
        
        let evaluate_alerts_use_case = Arc::new(EvaluateAlertsUseCase::new(
            alert_repo.clone(),
            alert_state_repo.clone(),
            alert_history_repo.clone(),
            event_bus.clone()
        ));

        // Alert use cases
        let create_alert_rule_use_case = Arc::new(CreateAlertRuleUseCase::new(alert_repo.clone()));
        let list_alert_rules_use_case = Arc::new(ListAlertRulesUseCase::new(alert_repo.clone()));
        let get_alert_rule_use_case = Arc::new(GetAlertRuleUseCase::new(alert_repo.clone()));
        let update_alert_rule_use_case = Arc::new(UpdateAlertRuleUseCase::new(alert_repo.clone()));
        let delete_alert_rule_use_case = Arc::new(DeleteAlertRuleUseCase::new(alert_repo.clone()));
        let list_alert_history_use_case = Arc::new(ListAlertHistoryUseCase::new(alert_history_repo.clone()));

        // Plugin infrastructure
        let modrinth_client = Arc::new(ModrinthClient::new());
        let search_plugins_use_case = Arc::new(SearchPluginsUseCase::new(modrinth_client.clone()));
        let get_plugin_versions_use_case = Arc::new(GetPluginVersionsUseCase::new(modrinth_client.clone()));
        let install_plugin_use_case = Arc::new(InstallPluginUseCase::new(repo.clone(), modrinth_client.clone()));

        // Template infrastructure
        let template_repository = Arc::new(SqlxTemplateRepository::new(pool.clone()));
        let template_repo: Arc<dyn TemplateRepository> = template_repository;

        let create_template_use_case = Arc::new(CreateTemplateUseCase::new(template_repo.clone()));
        let list_templates_use_case = Arc::new(ListTemplatesUseCase::new(template_repo.clone()));
        let get_template_use_case = Arc::new(GetTemplateUseCase::new(template_repo.clone()));
        let update_template_use_case = Arc::new(UpdateTemplateUseCase::new(template_repo.clone()));
        let delete_template_use_case = Arc::new(DeleteTemplateUseCase::new(template_repo.clone()));
        let apply_template_use_case = Arc::new(ApplyTemplateUseCase::new(template_repo.clone()));

        // Webhook infrastructure (Discord alerts)
        let discord_client = Arc::new(DiscordClient::new());
        let webhook_service = Arc::new(WebhookService::new(
            repo.clone(),
            discord_client.clone(),
            event_bus.clone(),
        ));

        // Backup infrastructure
        let backup_repository_concrete = Arc::new(PostgresBackupRepository::new(pool.clone()));
        let backup_repo: Arc<dyn BackupRepository> = backup_repository_concrete;
        let local_provider = Arc::new(LocalClient::new(std::path::PathBuf::from("/backups")));
        let s3_client = Arc::new(S3Client::new());
        let settings_repository_concrete = Arc::new(PostgresSettingsRepository::new(pool.clone()));
        let settings_repo: Arc<dyn SettingsRepository> = settings_repository_concrete;
        
        let backup_service = Arc::new(BackupService::new(
            repo.clone(),
            backup_repo.clone(),
            local_provider,
            s3_client,
            settings_repo.clone(),
        ));

    // Backup config repository
    let backup_config_repository_concrete = Arc::new(PostgresBackupConfigRepository::new(pool.clone()));
    let backup_config_repository: Arc<dyn BackupConfigRepository> = backup_config_repository_concrete;
    let backup_config_repo = backup_config_repository.clone();

    // Backup scheduler — kept for construction but spawn disabled in bootstrap/mod.rs (D-02)
    let backup_scheduler = Arc::new(BackupScheduler::new(
        backup_service.clone(),
        repo.clone(),
        backup_repo.clone(),
    ));

        // Phase 60: Crash Detection channel
        // The WS handler sends CrashReportData through this channel,
        // and the MonitoringService drains it at the top of each tick.
        let (crash_report_tx, crash_report_rx) = mpsc::channel::<CrashReportData>(256);

        let monitoring_service = Arc::new(MonitoringService::new(
            repo.clone(),
            metrics.clone(),
            factory.clone(),
            event_bus.clone(),
            evaluate_alerts_use_case.clone(),
            node_repo.clone(),
            pool.clone(),
            Some(crash_report_rx),
        ));

        let billing_service: Arc<dyn BillingService> = if config.lemon_squeezy_api_key.is_some() {
            Arc::new(crate::infrastructure::billing::LemonSqueezyService::new(
                config.lemon_squeezy_api_key.clone(),
                config.app_url.clone(),
            ))
        } else {
            tracing::warn!("No billing service configured - subscriptions will not work");
            Arc::new(crate::infrastructure::billing::LemonSqueezyService::new(None, config.app_url.clone()))
        };

        Self {
            pool: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
            app_url: config.app_url.clone(),
            billing_service,
            billing_webhook_secret: config.lemon_squeezy_webhook_secret,
            solys_client,
            create_server_use_case,
            list_servers_use_case,
            get_server_use_case,
            update_server_use_case,
            delete_server_use_case,
            start_server_use_case,
            stop_server_use_case,
            send_command_use_case,
            get_server_snapshot_use_case,
            evaluate_alerts_use_case,
            monitoring_service,
            event_bus,
            alert_repository: alert_repo,
            create_alert_rule_use_case,
            list_alert_rules_use_case,
            get_alert_rule_use_case,
            update_alert_rule_use_case,
            delete_alert_rule_use_case,
            list_alert_history_use_case,
            metrics_repository: metrics,
            search_plugins_use_case,
            get_plugin_versions_use_case,
            install_plugin_use_case,
            webhook_service,
            backup_service,
            backup_scheduler,
            settings_repository: settings_repo,
            server_repository: repo,
            node_repository: node_repo,
            node_metrics_repository: node_metrics_repo,
            node_api_key_repository: node_api_key_repo,
            node_registration_token_repository: node_registration_token_repo,
            node_connection_manager,
            node_client,
            node_health_service: Some(node_health_service),
            discord_client: discord_client.clone(),
            rbac_repository: rbac_repository,
            redis_pool,
            cron_task_repository,
            backup_config_repository: backup_config_repo,
            // Template use cases
            create_template_use_case,
            list_templates_use_case,
            get_template_use_case,
            update_template_use_case,
            delete_template_use_case,
            apply_template_use_case,
            crash_report_tx: Some(crash_report_tx),
        }
    }
}

