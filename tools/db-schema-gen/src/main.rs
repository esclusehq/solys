mod introspector;
mod rustdoc_reader;
mod domain_classifier;
mod mermaid_generator;
mod markdown_builder;
mod types;

use clap::Parser;
use sqlx::postgres::PgPoolOptions;

use types::{DomainSection, TableCluster, Relationship, TableInfo};

#[derive(Parser, Debug)]
#[command(name = "db-schema-gen", about = "Generate DATABASE_SCHEMA.md from PostgreSQL schema")]
struct Cli {
    #[arg(short, long, env = "DATABASE_URL")]
    db_url: String,

    #[arg(short, long, default_value = "DATABASE_SCHEMA.md")]
    output: String,

    #[arg(short, long, default_value = "api/src/domain")]
    source_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cli.db_url)
        .await?;

    let table_names = introspector::get_tables(&pool).await?;
    let enums = introspector::get_enums(&pool).await?;
    let annotations = rustdoc_reader::read_entity_docs(&cli.source_dir)?;

    let mut all_tables: Vec<TableInfo> = Vec::new();
    for name in &table_names {
        let columns = introspector::get_columns(&pool, name).await?;
        let foreign_keys = introspector::get_foreign_keys(&pool, name).await?;
        let indexes = introspector::get_indexes(&pool, name).await?;
        let table_comment = introspector::get_table_comment(&pool, name).await?;
        all_tables.push(TableInfo {
            table_name: name.clone(),
            table_comment,
            columns,
            foreign_keys,
            indexes,
        });
    }

    let domain_names: Vec<&str> = {
        let mut names: Vec<&str> = all_tables
            .iter()
            .map(|t| domain_classifier::classify_table(&t.table_name))
            .collect();
        names.sort();
        names.dedup();
        names
    };

    let descriptions = domain_classifier::get_domain_descriptions();
    let clusters = domain_classifier::get_relationship_clusters();
    let table_to_entity = domain_classifier::get_table_to_entity_map();

    let mut domains: Vec<DomainSection> = Vec::new();
    for domain_name in &domain_names {
        let domain_tables: Vec<TableInfo> = all_tables
            .iter()
            .filter(|t| domain_classifier::classify_table(&t.table_name) == *domain_name)
            .cloned()
            .collect();

        let domain_clusters: Vec<TableCluster> = clusters
            .iter()
            .filter(|(_, tables_in_cluster, _)| {
                tables_in_cluster
                    .iter()
                    .any(|t| domain_tables.iter().any(|dt| dt.table_name == *t))
            })
            .map(|(cluster_name, table_names_in_cluster, cluster_desc)| {
                let cluster_tables: Vec<TableInfo> = table_names_in_cluster
                    .iter()
                    .filter_map(|t| {
                        domain_tables
                            .iter()
                            .find(|dt| dt.table_name == *t)
                            .cloned()
                    })
                    .collect();

                let relationships: Vec<Relationship> = cluster_tables
                    .iter()
                    .flat_map(|t| {
                        t.foreign_keys.iter().filter_map(|fk| {
                            let parent_table = &fk.foreign_table_name;
                            let child_table = &t.table_name;
                            let parent_col = cluster_tables.iter().find(|ct| {
                                ct.columns.iter().any(|c| {
                                    c.name == fk.foreign_column_name && ct.table_name == *parent_table
                                })
                            });
                            let parent_nullable = parent_col
                                .and_then(|ct| {
                                    ct.columns
                                        .iter()
                                        .find(|c| c.name == fk.foreign_column_name)
                                })
                                .map(|c| c.is_nullable)
                                .unwrap_or(false);

                            let child_col = t.columns.iter().find(|c| c.name == fk.column_name);
                            let child_nullable = child_col.map(|c| c.is_nullable).unwrap_or(false);

                            let parent_cardinality = if parent_nullable {
                                "|o"
                            } else {
                                "||"
                            };
                            let child_cardinality = if child_nullable {
                                "o{"
                            } else {
                                "||"
                            };

                            Some(Relationship {
                                parent_table: parent_table.clone(),
                                child_table: child_table.clone(),
                                parent_cardinality: parent_cardinality.to_string(),
                                child_cardinality: child_cardinality.to_string(),
                                description: "references".to_string(),
                            })
                        })
                    })
                    .collect();

                TableCluster {
                    name: cluster_name.to_string(),
                    tables: cluster_tables,
                    relationships,
                }
            })
            .collect();

        let query_patterns = extract_query_patterns(&domain_name, &annotations);
        let design_rationale = extract_design_rationale(&domain_name, &annotations);

        let description = descriptions
            .get(*domain_name)
            .copied()
            .unwrap_or("")
            .to_string();

        domains.push(DomainSection {
            name: domain_name.to_string(),
            description,
            tables: domain_tables,
            clusters: domain_clusters,
            query_patterns,
            design_rationale,
        });
    }

    let document = markdown_builder::build_document(&domains);
    std::fs::write(&cli.output, document)?;
    println!("Generated {}", &cli.output);

    Ok(())
}

fn extract_query_patterns(
    _domain_name: &str,
    annotations: &[types::EntityDoc],
) -> Vec<String> {
    let mut patterns = Vec::new();
    for annotation in annotations {
        for line in &annotation.doc_lines {
            if line.to_lowercase().contains("query") || line.to_lowercase().contains("select") {
                patterns.push(line.clone());
            }
        }
    }
    patterns
}

fn extract_design_rationale(
    _domain_name: &str,
    annotations: &[types::EntityDoc],
) -> Vec<String> {
    let mut rationale = Vec::new();
    for annotation in annotations {
        for line in &annotation.doc_lines {
            if line.to_lowercase().contains("because")
                || line.to_lowercase().contains("purpose")
                || line.to_lowercase().contains("motivation")
            {
                rationale.push(line.clone());
            }
        }
    }
    rationale
}
