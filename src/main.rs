use anyhow::{Result, anyhow, bail};
use clap::Parser;
use cli::Args;
use exif::{In, Tag};
use std::{
    cmp::Ordering,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};

mod cli;

fn main() -> Result<()> {
    let args = Args::parse();

    let mut files = list_images(&args.dir)?;

    if !args.revert {
        files.sort_by(sort_by_time);
        if args.desc {
            files.reverse();
        }
    }

    let prefix_len = get_prefix_len(files.len());

    match (args.revert, args.test) {
        // Revert
        (true, false) => {
            for file in files.iter() {
                if let Err(e) = revert_file(file, args.delim.as_ref()) {
                    eprintln!("{e}");
                }
            }
        }
        // Revert, Test
        (true, true) => {
            for file in files.iter() {
                test_revert_file(file, args.delim.as_ref());
            }
        }
        // Rename
        (false, false) => {
            for (index, file) in files.iter().enumerate() {
                if let Err(e) = rename_file(file, index, prefix_len, args.delim.as_ref()) {
                    eprintln!("{e}");
                }
            }
        }
        // Rename, Test
        (false, true) => {
            for (index, file) in files.iter().enumerate() {
                test_rename_file(file, index, prefix_len, args.delim.as_ref());
            }
        }
    }

    Ok(())
}

fn list_images<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
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

fn test_rename_file(file: &Path, index: usize, prefix_len: usize, delim: &str) {
    let org = file.file_name().unwrap().to_string_lossy();
    let index = index + 1;
    let prefix = create_prefix(index, prefix_len);
    let new_name = format!("{prefix}{delim}{org}");

    println!("{org} -> {new_name}");
}

fn rename_file(file: &Path, index: usize, prefix_len: usize, delim: &str) -> Result<()> {
    let org = file.file_name().unwrap().to_string_lossy();
    let index = index + 1;
    let prefix = create_prefix(index, prefix_len);
    let new_name = format!("{prefix}{delim}{org}");

    let parent = file.parent().unwrap();
    let mut to = PathBuf::from(parent);
    to.push(new_name);

    fs::rename(file, &to)
        .map_err(|_| anyhow!("Failed to rename file {}", file.to_string_lossy()))?;

    println!(
        "Renamed: {} -> {}",
        file.file_name().unwrap().to_string_lossy(),
        to.file_name().unwrap().to_string_lossy()
    );

    Ok(())
}

fn test_revert_file(file: &Path, delim: &str) {
    let org = file.file_name().unwrap().to_string_lossy();

    match org.find(delim) {
        Some(prefix_index) => {
            let new_name = &org[prefix_index + 2..];
            println!("{org} -> {new_name}");
        }
        None => {
            println!("{org} is not renamed.");
        }
    }
}

fn revert_file(file: &Path, delim: &str) -> Result<()> {
    let org = file.file_name().unwrap().to_string_lossy();

    let Some(prefix_index) = org.find(delim) else {
        println!("Not processed: {}", org);
        return Ok(());
    };
    let new_name = &org[prefix_index + 2..];

    let parent = file.parent().unwrap();
    let mut to = PathBuf::from(parent);
    to.push(new_name);

    fs::rename(file, &to)
        .map_err(|_| anyhow!("Failed to revert file name {}", file.to_string_lossy()))?;

    println!(
        "Reverted: {} -> {}",
        file.file_name().unwrap().to_string_lossy(),
        to.file_name().unwrap().to_string_lossy()
    );

    Ok(())
}

fn get_prefix_len(files_len: usize) -> usize {
    let files_len = files_len.to_string();
    files_len.len()
}

fn create_prefix(num: usize, len: usize) -> String {
    let num = num.to_string();

    if len < num.len() {
        num
    } else {
        let mut prefix = String::new();
        for _i in 0..(len - num.len()) {
            prefix.push('0');
        }
        prefix.push_str(&num);
        prefix
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_prefix() {
        let actual = create_prefix(10, 3);
        assert_eq!(actual, String::from("010"));

        let actual = create_prefix(5, 3);
        assert_eq!(actual, String::from("005"));

        let actual = create_prefix(100, 2);
        assert_eq!(actual, String::from("100"));
    }
}
