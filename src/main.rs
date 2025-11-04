use anyhow::{Result, bail};
use clap::Parser;
use cli::Args;
use std::{fs, path::Path};

use crate::file::PhotoFile;

mod cli;
mod file;

fn main() -> Result<()> {
    let args = Args::parse();

    let files = list_images(&args.dir, args.desc)?;

    let prefix_len = get_prefix_len(files.len());
    let delim = args.delim.as_ref();

    match (args.revert, args.test) {
        // Revert
        (true, false) => {
            for file in files.iter() {
                if let Err(e) = file.revert_name(delim) {
                    eprintln!("{e}");
                }
            }
        }
        // Revert, Test
        (true, true) => {
            for file in files.iter() {
                let org = file.get_name();
                match file.create_reverted_name(delim) {
                    Some(new_name) => println!("{org} -> {new_name}"),
                    None => println!("{org} is not renamed."),
                }
            }
        }
        // Rename
        (false, false) => {
            for (index, file) in files.iter().enumerate() {
                if let Err(e) = file.rename_with_prefix(index, prefix_len, delim) {
                    eprintln!("{e}");
                }
            }
        }
        // Rename, Test
        (false, true) => {
            for (index, file) in files.iter().enumerate() {
                let org = file.get_name();
                let new_name = file.create_prefixed_name(index, prefix_len, delim);
                println!("{org} -> {new_name}");
            }
        }
    }

    Ok(())
}

fn list_images<P: AsRef<Path>>(root: P, desc: bool) -> Result<Vec<PhotoFile>> {
    let files = match fs::read_dir(root) {
        Ok(files) => files.filter_map(|file| file.ok()),
        Err(_) => bail!("Failed to list files."),
    };

    let mut images = files
        .map(|file| file.path())
        .filter_map(|path| {
            if PhotoFile::is_supported_file(&path) {
                Some(PhotoFile::from(path))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    images.sort();
    if desc {
        images.reverse();
    }

    Ok(images)
}

fn get_prefix_len(files_len: usize) -> usize {
    let files_len = files_len.to_string();
    files_len.len()
}
