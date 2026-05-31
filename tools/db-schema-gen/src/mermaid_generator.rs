use crate::types::TableCluster;

pub fn generate_er_diagram(cluster: &TableCluster) -> String {
    let mut output = String::from("```mermaid\nerDiagram\n");

    for table in &cluster.tables {
        output.push_str(&format!("    {} {{\n", table.table_name));
        for col in &table.columns {
            let mut markers = String::new();
            if col.is_pk {
                markers.push_str("PK");
            }
            if col.is_fk {
                if !markers.is_empty() {
                    markers.push(' ');
                }
                markers.push_str("FK");
            }
            if markers.is_empty() {
                output.push_str(&format!("        {} {}\n", col.data_type, col.name));
            } else {
                output.push_str(&format!(
                    "        {} {} {}\n",
                    col.data_type, col.name, markers
                ));
            }
        }
        output.push_str("    }\n");
    }

    for rel in &cluster.relationships {
        output.push_str(&format!(
            "    {} {} {} {} : {}\n",
            rel.parent_table,
            rel.parent_cardinality,
            rel.child_table,
            rel.child_cardinality,
            rel.description
        ));
    }

    output.push_str("```\n");
    output
}

pub fn validate_er_diagram(diagram: &str) -> String {
    if diagram.contains("{}") {
        let mut result = String::new();
        let mut skip = false;
        for line in diagram.lines() {
            if line.trim() == "{" {
                skip = true;
            } else if skip && line.trim() == "}" {
                skip = false;
                continue;
            }
            if !skip {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    } else {
        diagram.to_string()
    }
}
