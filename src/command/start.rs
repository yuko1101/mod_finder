use std::{collections::HashSet, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::util::{self, mod_parser::ModFile};

pub fn start(mods_dir: PathBuf, consistent_mods: String) -> Result<()> {
    println!("Starting with mods_dir: {:?}", mods_dir);

    let consistent_mods: Vec<String> = consistent_mods.split(',').map(|s| s.to_string()).collect();

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

    bisect(
        &mods_dir,
        &mod_files.iter().collect(),
        &consistent_mods,
        &builtin_mods,
    )?;

    Ok(())
}

fn bisect(
    mods_dir: &PathBuf,
    mod_files: &Vec<&ModFile>,
    consistent_mods: &Vec<String>,
    builtin_mods: &HashSet<String>,
) -> Result<()> {
    let half = take_half(mod_files, consistent_mods, builtin_mods)?;
    println!(
        "Half of mods: {:?}",
        half.iter().map(|m| m.file_name.clone()).collect::<Vec<_>>()
    );

    let mods_disabled_dir = mods_dir.parent().unwrap().join("mods.disabled");
    if !mods_disabled_dir.exists() {
        std::fs::create_dir(&mods_disabled_dir)?;
    }

    let mut disabled_mods = HashSet::new();
    for &mod_file in mod_files {
        if half.contains(mod_file) {
            continue;
        }

        let file_name = &mod_file.file_name;
        std::fs::rename(
            &mods_dir.join(file_name),
            &mods_disabled_dir.join(file_name),
        )?;
        disabled_mods.insert(mod_file);
    }

    // bisect check
    let bisect_result = dialoguer::Confirm::new()
        .with_prompt("The mod you want to find is still enabled?")
        .interact()?;

    println!("Bisect result: {}", bisect_result);

    if !bisect_result {
        // swap mods and disabled mods
        for &mod_file in &disabled_mods {
            std::fs::rename(
                &mods_disabled_dir.join(&mod_file.file_name),
                &mods_dir.join(&mod_file.file_name),
            )?;
        }
        for &mod_file in &half {
            std::fs::rename(
                &mods_dir.join(&mod_file.file_name),
                &mods_disabled_dir.join(&mod_file.file_name),
            )?;
        }
    }

    let new_mod_files: Vec<&ModFile> = (if bisect_result {
        half
    } else {
        let mut extra_dependencies = HashSet::new();
        for mod_file in &disabled_mods {
            extra_dependencies.extend(mod_file.get_extra_dependencies(
                mod_files,
                &disabled_mods,
                builtin_mods,
            )?);
        }

        for dependency in &extra_dependencies {
            if !disabled_mods.contains(dependency) {
                std::fs::rename(
                    &mods_disabled_dir.join(&dependency.file_name),
                    &mods_dir.join(&dependency.file_name),
                )?;
            }
        }

        &disabled_mods | &extra_dependencies
    })
    .iter()
    .cloned()
    .collect();

    if new_mod_files.len() == 1 {
        return Ok(());
    }
    bisect(mods_dir, &new_mod_files, consistent_mods, builtin_mods)
}

fn take_half<'a>(
    mod_files: &'a Vec<&ModFile>,
    consistent_mods: &Vec<String>,
    builtin_mods: &HashSet<String>,
) -> Result<HashSet<&'a ModFile>> {
    let mut half = HashSet::new();
    let mut i = 0;

    for const_mod in consistent_mods {
        if let Some(&mod_file) = mod_files
            .iter()
            .find(|m| m.get_mod_ids().contains(&const_mod))
        {
            half.insert(mod_file);
            half.extend(mod_file.get_extra_dependencies(mod_files, &half, builtin_mods)?);
        } else {
            return Err(anyhow!("Missing consistent mod: {}", const_mod));
        }
    }

    while half.len() < mod_files.len() / 2 {
        let mod_file = mod_files[i];
        half.insert(mod_file);
        half.extend(mod_file.get_extra_dependencies(mod_files, &half, builtin_mods)?);
        i += 1;
    }

    Ok(half)
}
