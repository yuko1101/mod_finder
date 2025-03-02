use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;

use crate::util::{self, mod_parser::ModFile};

pub fn start(mods_dir: PathBuf) -> Result<()> {
    println!("Starting with mods_dir: {:?}", mods_dir);

    let mut builtin_mods = HashSet::new();
    builtin_mods.insert("minecraft".to_string());
    builtin_mods.insert("neoforge".to_string());

    let mut mod_files = Vec::new();
    for file in mods_dir.read_dir()? {
        let file = file?;
        let mod_file = util::mod_parser::parse_mod(file.path());
        let Ok(mod_file) = mod_file else {
            let err = mod_file.unwrap_err();
            eprintln!(
                "Error parsing {} mod: {}",
                file.file_name().to_string_lossy(),
                err
            );
            continue;
        };

        println!("Found mod: {:?}", mod_file);
        mod_files.push(mod_file);
    }

    let half = take_half(&mod_files, &builtin_mods)?;
    println!("Half of mods: {:?}", half);

    Ok(())
}

fn take_half<'a>(
    mod_files: &'a Vec<ModFile>,
    builtin_mods: &HashSet<String>,
) -> Result<HashSet<&'a ModFile>> {
    let mut half = HashSet::new();
    let mut i = 0;
    while half.len() < mod_files.len() / 2 {
        let mod_file = &mod_files[i];
        half.insert(mod_file);
        half.extend(mod_file.get_dependencies(mod_files, builtin_mods)?);
        i += 1;
    }

    Ok(half)
}
