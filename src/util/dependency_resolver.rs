use std::collections::HashSet;

use anyhow::{Result, anyhow};

use super::mod_parser::ModFile;

impl ModFile {
    pub fn get_mod_ids(&self) -> Vec<&String> {
        self.meta_list.iter().map(|m| &m.id).collect()
    }

    pub fn get_extra_dependencies<'a>(
        &self,
        mods: &'a Vec<ModFile>,
        current_mods: &HashSet<&'a ModFile>,
        builtin_mods: &HashSet<String>,
    ) -> Result<HashSet<&'a ModFile>> {
        let mut dependencies = HashSet::new();
        for mod_meta in &self.meta_list {
            for dependency in mod_meta.dependencies.iter() {
                if self.get_mod_ids().contains(&dependency) || builtin_mods.contains(dependency) {
                    continue;
                }
                if let Some(_) = current_mods
                    .iter()
                    .find(|m| m.get_mod_ids().contains(&dependency))
                {
                    continue;
                }

                if let Some(mod_file) = mods.iter().find(|m| m.get_mod_ids().contains(&dependency))
                {
                    dependencies.insert(mod_file);
                } else {
                    return Err(anyhow!("Missing dependency: {}", dependency));
                }
            }
        }
        Ok(dependencies)
    }
}
