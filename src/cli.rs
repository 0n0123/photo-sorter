use std::{path::{Path, PathBuf}, str::FromStr};

use anyhow::bail;
use clap::Parser;

#[derive(Clone)]
pub struct DirPath(PathBuf);

#[derive(Clone)]
pub struct Delim(String);

#[derive(Parser)]
pub struct Args {
    /// Path to directory includes photos
    pub dir: DirPath,
    /// Prefix delimiter
    #[clap(short, long, default_value = "__")]
    pub delim: Delim,
    #[clap(short, long, default_value = "false")]
    /// Test mode that only shows order
    pub test: bool,
    /// Sorts latest to oldest order
    #[clap(long, default_value = "false")]
    pub desc: bool,
    /// Revert renamed files
    #[clap(short, long, default_value = "false")]
    pub revert: bool,
}

impl FromStr for DirPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(s);
        if !path.exists() {
            bail!("Path {s} is not found.")
        }
        if !path.is_dir() {
            bail!("Path {s} is not a folder.")
        }

        Ok(Self(path))
    }
}

impl AsRef<Path> for DirPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl FromStr for Delim {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            bail!("Delimiter is too short.");
        }

        Ok(Self(String::from(s)))
    }
}

impl AsRef<str> for Delim {
    fn as_ref(&self) -> &str {
        &self.0
    }
}