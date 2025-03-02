use std::path::PathBuf;

use anyhow::Result;

use crate::util;

pub fn start(mods_dir: PathBuf) -> Result<()> {
    println!("Starting with mods_dir: {:?}", mods_dir);
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
    }

    Ok(())
}
