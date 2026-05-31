#[derive(Debug, Clone)]
pub struct TableInfo {
    pub table_name: String,
    pub table_comment: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub foreign_keys: Vec<ForeignKey>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub ordinal_position: i32,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_pk: bool,
    pub is_fk: bool,
    pub default: Option<String>,
    pub description: Option<String>,
    pub max_length: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub column_name: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
    pub update_rule: String,
    pub delete_rule: String,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub index_name: String,
    pub index_definition: String,
}

#[derive(Debug, Clone)]
pub struct EntityDoc {
    pub struct_name: String,
    pub doc_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DomainSection {
    pub name: String,
    pub description: String,
    pub tables: Vec<TableInfo>,
    pub clusters: Vec<TableCluster>,
    pub query_patterns: Vec<String>,
    pub design_rationale: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TableCluster {
    pub name: String,
    pub tables: Vec<TableInfo>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    pub parent_table: String,
    pub child_table: String,
    pub parent_cardinality: String,
    pub child_cardinality: String,
    pub description: String,
}
