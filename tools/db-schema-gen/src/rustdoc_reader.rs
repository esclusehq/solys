use regex::Regex;
use walkdir::WalkDir;

use crate::types::EntityDoc;

pub fn read_entity_docs(source_dir: &str) -> anyhow::Result<Vec<EntityDoc>> {
    let re_doc = Regex::new(r"^\s*///\s?(.*)$")?;
    let re_struct = Regex::new(r"pub struct (\w+)")?;
    let mut entities = Vec::new();

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".rs"))
    {
        let content = std::fs::read_to_string(entry.path())?;
        let mut current_doc: Vec<String> = Vec::new();
        let mut current_struct: Option<String> = None;

        for line in content.lines() {
            if let Some(cap) = re_doc.captures(line) {
                current_doc.push(cap[1].to_string());
            } else if let Some(cap) = re_struct.captures(line) {
                current_struct = Some(cap[1].to_string());
                if !current_doc.is_empty() {
                    if let Some(ref name) = current_struct {
                        entities.push(EntityDoc {
                            struct_name: name.clone(),
                            doc_lines: current_doc.clone(),
                        });
                    }
                }
                current_doc.clear();
            } else {
                current_doc.clear();
            }
        }
    }

    Ok(entities)
}
