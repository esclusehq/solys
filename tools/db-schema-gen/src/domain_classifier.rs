use std::collections::HashMap;

pub fn classify_table(table_name: &str) -> &'static str {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert("servers", "Servers");
    map.insert("server_events", "Servers");
    map.insert("server_metrics", "Servers");
    map.insert("server_crash_logs", "Servers");
    map.insert("nodes", "Nodes");
    map.insert("node_metrics", "Nodes");
    map.insert("node_events", "Nodes");
    map.insert("server_nodes", "Nodes");
    map.insert("users", "Users/Auth");
    map.insert("api_keys", "Users/Auth");
    map.insert("roles", "Users/Auth");
    map.insert("permissions", "Users/Auth");
    map.insert("user_roles", "Users/Auth");
    map.insert("plans", "Billing/Subscriptions");
    map.insert("subscriptions", "Billing/Subscriptions");
    map.insert("billing_customers", "Billing/Subscriptions");
    map.insert("payment_transactions", "Billing/Subscriptions");
    map.insert("invoices", "Billing/Subscriptions");
    map.insert("refunds", "Billing/Subscriptions");
    map.insert("backup_history", "Backups");
    map.insert("s3_profiles", "Backups");
    map.insert("app_settings", "Settings/Config");
    map.insert("cron_tasks", "Settings/Config");
    map.insert("audit_logs", "Events/Logs");
    map.insert("alert_rules", "Events/Logs");
    map.insert("alert_states", "Events/Logs");
    map.insert("alert_history", "Events/Logs");
    map.insert("jobs", "Jobs");
    map.insert("templates", "Templates");
    map.insert("modpack_templates", "Templates");
    map.insert("plugin_templates", "Templates");
    map.insert("game_types", "Games");
    map.insert("port_pools", "Games");
    map.insert("resource_plans", "Games");
    map.insert("deployment_configs", "Games");
    map.insert("webhooks", "Webhooks");
    map.insert("usage_tracking", "Usage");
    map.insert("agents", "Infrastructure");
    map.insert("node_api_keys", "Infrastructure");
    map.insert("node_registration_tokens", "Infrastructure");
    map.get(table_name).copied().unwrap_or("Other")
}

pub fn get_domain_descriptions() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert(
        "Servers",
        "Game server records with connection details, runtime configuration, and deployment snapshots. Central entity for all server lifecycle management.",
    );
    map.insert(
        "Nodes",
        "Compute node registration and monitoring. Each node runs the Solys Web Agent and can host multiple game server containers.",
    );
    map.insert(
        "Users/Auth",
        "User identity, authentication, and role-based access control. Manages accounts, API keys, permissions, and authorization.",
    );
    map.insert(
        "Billing/Subscriptions",
        "Billing plans, user subscriptions, payment processing, invoicing, and refund management. Integrates with Stripe/LemonSqueezy.",
    );
    map.insert(
        "Backups",
        "Backup history records and S3-compatible storage profiles for server backup operations.",
    );
    map.insert(
        "Settings/Config",
        "Platform-wide configuration stored in app_settings key-value table and scheduled cron task definitions.",
    );
    map.insert(
        "Events/Logs",
        "Audit trail for security events and threshold-based alert rules for server and node monitoring.",
    );
    map.insert(
        "Jobs",
        "Background asynchronous job queue with progress tracking, used for backups, plugin installation, and other async operations.",
    );
    map.insert(
        "Templates",
        "Pre-configured server templates, modpack references, and plugin bundles for quick server deployment.",
    );
    map.insert(
        "Games",
        "Game type definitions, port pools, resource allocation plans, and deployment configurations.",
    );
    map.insert(
        "Webhooks",
        "Outgoing webhook configurations for event-driven integrations with external services.",
    );
    map.insert(
        "Usage",
        "Metered usage tracking per billing period for resource consumption monitoring and quota enforcement.",
    );
    map.insert(
        "Infrastructure",
        "Legacy agent records, node API key management, and one-time registration tokens for automated node enrollment.",
    );
    map
}

pub fn get_relationship_clusters() -> Vec<(&'static str, Vec<&'static str>, &'static str)> {
    vec![
        (
            "Users & Auth",
            vec!["users", "api_keys", "roles", "permissions", "user_roles"],
            "User identity, role-based access control, and API key management",
        ),
        (
            "Servers & Events",
            vec![
                "servers",
                "server_events",
                "server_metrics",
                "server_crash_logs",
                "cron_tasks",
            ],
            "Game server records with event logs, metrics snapshots, crash forensic data, and scheduled tasks",
        ),
        (
            "Nodes & Infrastructure",
            vec![
                "nodes",
                "node_metrics",
                "node_events",
                "node_api_keys",
                "node_registration_tokens",
                "server_nodes",
            ],
            "Compute node registration, monitoring, authentication, and server-node assignments",
        ),
        (
            "Billing & Subscriptions",
            vec![
                "plans",
                "subscriptions",
                "billing_customers",
                "payment_transactions",
                "invoices",
                "refunds",
            ],
            "Billing plans, user subscriptions, payment processing, and refund tracking",
        ),
        (
            "Backups & Storage",
            vec!["backup_history", "s3_profiles"],
            "Backup records and S3-compatible storage profiles",
        ),
        (
            "Alerts & Monitoring",
            vec!["alert_rules", "alert_states", "alert_history"],
            "Threshold-based alert rules, state tracking, and historical alert events",
        ),
        (
            "Jobs",
            vec!["jobs"],
            "Background async job queue with progress tracking",
        ),
        (
            "Webhooks & Usage",
            vec!["webhooks", "usage_tracking"],
            "Outgoing webhook configurations and metered usage tracking",
        ),
        (
            "Game Configuration",
            vec!["game_types", "deployment_configs", "port_pools", "resource_plans"],
            "Game type definitions, deployment configurations, port pools, and resource allocation plans",
        ),
        (
            "Templates",
            vec!["templates", "modpack_templates", "plugin_templates"],
            "Pre-configured server templates, modpack references, and plugin bundles",
        ),
    ]
}

pub fn get_table_to_entity_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("backup_history", "BackupRecord");
    map.insert("app_settings", "AppSettings");
    map.insert("usage_tracking", "UsageRecord");
    map.insert("server_nodes", "ServerNode");
    map
}
