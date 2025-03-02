use std::{
    fs::File,
    hash::Hash,
    io::{Cursor, Read},
    path::PathBuf,
    vec,
};

use anyhow::{Ok, Result, anyhow};
use zip::result::ZipResult;

#[derive(Debug)]
pub struct ModFile {
    pub file_name: String,
    pub meta_list: Vec<ModMetadata>,
}

impl PartialEq for ModFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}

impl Eq for ModFile {}
impl Hash for ModFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_name.hash(state);
    }
}

#[derive(Debug)]
pub struct ModMetadata {
    pub name: String,
    pub id: String,
    pub dependencies: Vec<String>,
}

pub fn parse_mod(path: PathBuf) -> Result<ModFile> {
    if path
        .extension()
        .ok_or_else(|| anyhow!("Missing extension"))?
        != "jar"
    {
        return Err(anyhow!("Invalid file extension: {:?}", path));
    }

    let file = File::open(&path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut metadata_contents = vec![];
    {
        let mut meta_file = archive.by_name("META-INF/neoforge.mods.toml")?;
        let mut meta_content = String::new();
        meta_file.read_to_string(&mut meta_content)?;

        metadata_contents.push(meta_content);
    };

    let jarjar_content = {
        let result = archive.by_name("META-INF/jarjar/metadata.json");
        if let ZipResult::Ok(mut file) = result {
            let mut jarjar_content = String::new();
            file.read_to_string(&mut jarjar_content)?;
            Some(jarjar_content)
        } else {
            None
        }
    };
    if let Some(jarjar_content) = jarjar_content {
        let value: serde_json::Value = serde_json::from_str(&jarjar_content)?;
        let jar_paths = value
            .get("jars")
            .ok_or_else(|| anyhow!("Missing jars property in metadata.json"))?
            .as_array()
            .ok_or_else(|| anyhow!("jars is not an array"))?
            .iter()
            .map(|v| {
                v.get("path")
                    .ok_or_else(|| anyhow!("Missing path property in jar"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("path is not a string"))
                    .map(|s| s.to_string())
            });

        for path in jar_paths {
            let path = path?;
            let inner_zip = archive.by_name(&path)?;
            let inner_zip_bytes = inner_zip.bytes().collect::<Result<Vec<u8>, _>>()?; // TODO: don't fully read into memory
            let reader = Cursor::new(inner_zip_bytes);
            let mut inner_archive = zip::ZipArchive::new(reader)?;

            if let ZipResult::Ok(mut meta_file) =
                inner_archive.by_name("META-INF/neoforge.mods.toml")
            {
                let mut content = String::new();
                meta_file.read_to_string(&mut content)?;

                metadata_contents.push(content);
            }
        }
    }

    let mut meta_list = vec![];
    for content in metadata_contents {
        let meta = parse_neoforge_meta(&content)?;
        meta_list.extend(meta);
    }

    Ok(ModFile {
        file_name: path.file_name().unwrap().to_string_lossy().to_string(),
        meta_list,
    })
}

pub fn parse_neoforge_meta(meta_content: &str) -> Result<Vec<ModMetadata>> {
    let value: toml::Table = toml::from_str(meta_content)?;
    let mods_array = value
        .get("mods")
        .ok_or_else(|| anyhow!("Missing mods property in neoforge.mods.toml"))?
        .as_array()
        .ok_or_else(|| anyhow!("Mods is not an array"))?;

    let dependencies_map = value
        .get("dependencies")
        .ok_or_else(|| anyhow!("Missing dependencies property in neoforge.mods.toml"))?
        .as_table()
        .ok_or_else(|| anyhow!("Dependencies is not a table"))?;

    let mut mods_meta = vec![];
    for mod_value in mods_array {
        let id = mod_value
            .get("modId")
            .ok_or_else(|| anyhow!("Missing modId"))?
            .as_str()
            .ok_or_else(|| anyhow!("modId is not a string"))?;

        let empty_vec = vec![];
        let dependencies = dependencies_map
            .get(id)
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec);

        let required_dependencies = dependencies.iter().filter(|d| {
            d.get("type").is_some_and(|t| {
                t.as_str()
                    .is_some_and(|t| t.to_ascii_lowercase() == "required")
            })
        });

        let metadata = ModMetadata {
            name: mod_value
                .get("displayName")
                .ok_or_else(|| anyhow!("Missing displayName"))?
                .as_str()
                .ok_or_else(|| anyhow!("displayName is not a string"))?
                .to_string(),
            id: id.to_string(),
            dependencies: required_dependencies
                .filter_map(|d| {
                    d.get("modId")
                        .and_then(|m| m.as_str().map(|m| m.to_string()))
                })
                .collect::<Vec<_>>(),
        };

        mods_meta.push(metadata);
    }

    Ok(mods_meta)
}
