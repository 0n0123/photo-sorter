use anyhow::{anyhow, bail, Result};
use exif::{In, Tag};
use std::{
    cmp::Ordering,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Path to directory includes photos
    dir: PathBuf,
    #[clap(short, long, default_value = "false")]
    /// Test mode that only shows order
    test: bool,
    /// Sorts latest to oldest order
    #[clap(long, default_value = "false")]
    desc: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut files = list_images(&args.dir)?;
    files.sort_by(sort_by_time);
    if args.desc {
        files.reverse();
    }

    if args.test {
        for (index, file) in files.iter().enumerate() {
            println!(
                "{:03} | {}",
                index,
                file.file_name().unwrap().to_string_lossy()
            );
        }
    } else {
        for (index, file) in files.iter().enumerate() {
            if let Err(e) = rename_file(file, index) {
                eprintln!("{e}");
            }
        }
    }

    Ok(())
}

fn list_images(root: &Path) -> Result<Vec<PathBuf>> {
    let files = match fs::read_dir(root) {
        Ok(files) => files.filter_map(|file| file.ok()),
        Err(_) => bail!("Failed to list files."),
    };
    let mut images = Vec::new();
    for file in files {
        let path = file.path();
        if path.is_dir() {
            continue;
        }
        if path
            .extension()
            .map(|ext| ext.to_ascii_lowercase())
            .is_some_and(|ext| ext == "jpg" || ext == "jpeg" || ext == "heic" || ext == "heif")
        {
            images.push(path);
        }
    }

    Ok(images)
}

fn sort_by_time(f1: &PathBuf, f2: &PathBuf) -> Ordering {
    let f1 = fs::File::open(f1);
    let f2 = fs::File::open(f2);
    let (f1, f2) = match (f1, f2) {
        (Ok(f1), Ok(f2)) => (f1, f2),
        _ => return Ordering::Equal,
    };

    let mut r1 = BufReader::new(f1);
    let mut r2 = BufReader::new(f2);

    let exif1 = exif::Reader::new().read_from_container(&mut r1);
    let exif2 = exif::Reader::new().read_from_container(&mut r2);

    match (exif1, exif2) {
        (Ok(e1), Ok(e2)) => {
            let time1 = e1.get_field(Tag::DateTimeOriginal, In::PRIMARY);
            let time2 = e2.get_field(Tag::DateTimeOriginal, In::PRIMARY);
            match (time1, time2) {
                (Some(t1), Some(t2)) => t1
                    .display_value()
                    .to_string()
                    .cmp(&t2.display_value().to_string()),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            }
        }
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn rename_file(file: &Path, index: usize) -> Result<()> {
    let org = file.file_name().unwrap().to_string_lossy();
    let index = index + 1;
    let new_name = format!("{index:03}__{org}");

    let parent = file.parent().unwrap();
    let mut to = PathBuf::from(parent);
    to.push(new_name);

    fs::rename(file, &to)
        .map_err(|_| anyhow!("Failed to rename file {}", file.to_string_lossy()))?;
    
    println!(
        "Renamed: {} -> {}",
        file.file_name().unwrap().to_string_lossy(),
        to.to_string_lossy()
    );
    Ok(())
}
