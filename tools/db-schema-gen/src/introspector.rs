use std::collections::HashMap;

use sqlx::PgPool;

use crate::types::{ColumnInfo, ForeignKey, IndexInfo};

pub async fn get_tables(pool: &PgPool) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn get_columns(pool: &PgPool, table: &str) -> anyhow::Result<Vec<ColumnInfo>> {
    let rows = sqlx::query(
        r#"
        SELECT
            c.column_name,
            c.ordinal_position,
            c.data_type,
            c.is_nullable,
            c.character_maximum_length,
            c.column_default,
            pgd.description AS column_comment,
            EXISTS (
                SELECT 1 FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name
                WHERE tc.constraint_type = 'PRIMARY KEY'
                  AND tc.table_name = c.table_name
                  AND tc.table_schema = c.table_schema
                  AND ku.column_name = c.column_name
            ) AS is_pk,
            EXISTS (
                SELECT 1 FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name
                WHERE tc.constraint_type = 'FOREIGN KEY'
                  AND tc.table_name = c.table_name
                  AND tc.table_schema = c.table_schema
                  AND ku.column_name = c.column_name
            ) AS is_fk
        FROM information_schema.columns c
        LEFT JOIN pg_catalog.pg_statio_all_tables st
            ON st.schemaname = c.table_schema AND st.relname = c.table_name
        LEFT JOIN pg_catalog.pg_description pgd
            ON pgd.objoid = st.relid AND pgd.objsubid = c.ordinal_position::integer
        WHERE c.table_schema = 'public' AND c.table_name = $1
        ORDER BY c.ordinal_position
        "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await?;

    let mut columns = Vec::new();
    for row in rows {
        use sqlx::Row;
        let name: String = row.try_get("column_name")?;
        let ordinal_position: i32 = row.try_get("ordinal_position")?;
        let data_type_raw: String = row.try_get("data_type")?;
        let is_nullable_str: String = row.try_get("is_nullable")?;
        let max_length: Option<i32> = row.try_get("character_maximum_length").ok().flatten();
        let default: Option<String> = row.try_get("column_default").ok().flatten();
        let description: Option<String> = row.try_get("column_comment").ok().flatten();
        let is_pk: bool = row.try_get("is_pk")?;
        let is_fk: bool = row.try_get("is_fk")?;

        columns.push(ColumnInfo {
            name,
            ordinal_position,
            data_type: map_pg_type(&data_type_raw).to_string(),
            is_nullable: is_nullable_str == "YES",
            is_pk,
            is_fk,
            default,
            description,
            max_length,
        });
    }
    Ok(columns)
}

pub fn map_pg_type(pg_type: &str) -> String {
    match pg_type {
        "uuid" => "uuid".to_string(),
        "text" | "character varying" | "character" => "text".to_string(),
        "integer" | "bigint" | "smallint" => "integer".to_string(),
        "boolean" => "bool".to_string(),
        "real" | "double precision" | "numeric" => "real".to_string(),
        "timestamp with time zone" => "timestamptz".to_string(),
        "timestamp without time zone" => "timestamp".to_string(),
        "jsonb" | "json" => "jsonb".to_string(),
        "USER-DEFINED" => "enum".to_string(),
        _ => pg_type.to_lowercase(),
    }
}

pub async fn get_foreign_keys(pool: &PgPool, table: &str) -> anyhow::Result<Vec<ForeignKey>> {
    let rows = sqlx::query(
        r#"
        SELECT
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name,
            rc.update_rule,
            rc.delete_rule
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.referential_constraints rc
            ON tc.constraint_name = rc.constraint_name
        JOIN information_schema.constraint_column_usage ccu
            ON rc.unique_constraint_name = ccu.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = $1
        "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await?;

    let mut fks = Vec::new();
    for row in rows {
        use sqlx::Row;
        fks.push(ForeignKey {
            column_name: row.try_get("column_name")?,
            foreign_table_name: row.try_get("foreign_table_name")?,
            foreign_column_name: row.try_get("foreign_column_name")?,
            update_rule: row.try_get("update_rule")?,
            delete_rule: row.try_get("delete_rule")?,
        });
    }
    Ok(fks)
}

pub async fn get_indexes(pool: &PgPool, table: &str) -> anyhow::Result<Vec<IndexInfo>> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = 'public' AND tablename = $1 ORDER BY indexname",
    )
    .bind(table)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(index_name, index_definition)| IndexInfo {
            index_name,
            index_definition,
        })
        .collect())
}

pub async fn get_enums(pool: &PgPool) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT t.typname, e.enumlabel FROM pg_type t JOIN pg_enum e ON t.oid = e.enumtypid ORDER BY t.typname, e.enumsortorder",
    )
    .fetch_all(pool)
    .await?;

    let mut enums: HashMap<String, Vec<String>> = HashMap::new();
    for (typname, enumlabel) in rows {
        enums.entry(typname).or_default().push(enumlabel);
    }
    Ok(enums)
}

pub async fn get_table_comment(pool: &PgPool, table: &str) -> anyhow::Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT COALESCE(pgd.description, '') FROM pg_catalog.pg_class pgc LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = pgc.oid AND pgd.objsubid = 0 WHERE pgc.relname = $1",
    )
    .bind(table)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| {
        let s = r.0;
        if s.is_empty() { None } else { Some(s) }
    }))
}
