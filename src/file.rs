use std::{
    borrow::Cow,
    cmp::Ordering,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhotoFile(PathBuf);

impl PhotoFile {
    pub fn is_supported_file(path: &Path) -> bool {
        if path.is_dir() {
            return false;
        }
        match path.extension() {
            Some(ext) => {
                let ext = ext.to_string_lossy().to_lowercase();
                matches!(ext.as_str(), "jpg" | "jpeg" | "heic" | "heif")
            }
            None => false,
        }
    }

    /// 現在のファイル名を取得
    pub fn get_name(&self) -> Cow<'_, str> {
        self.0
            .file_name()
            .expect("File should have a name.")
            .to_string_lossy()
    }

    /// 番号付きの新しいファイル名を作成
    ///
    /// # Param
    /// - `index`: ファイル先頭に付加するインデックスの値(0始まり)
    /// - `prefix_len`: インデックスの桁数
    /// - `delim`: インデックスと元のファイル名の区切り文字列
    pub fn create_prefixed_name(&self, index: usize, prefix_len: usize, delim: &str) -> String {
        let org = self
            .0
            .file_name()
            .expect("File should have a name.")
            .to_string_lossy();
        let index = index + 1;
        let prefix = create_prefix(index, prefix_len);
        format!("{prefix}{delim}{org}")
    }

    /// ファイル名を変更
    ///
    /// # Param
    /// - `index`: ファイル先頭に付加するインデックスの値(0始まり)
    /// - `prefix_len`: インデックスの桁数
    /// - `delim`: インデックスと元のファイル名の区切り文字列
    pub fn rename_with_prefix(&self, index: usize, prefix_len: usize, delim: &str) -> Result<()> {
        let new_name = self.create_prefixed_name(index, prefix_len, delim);

        let parent = self.0.parent().expect("File should have a parent.");
        let mut to = PathBuf::from(parent);
        to.push(new_name);

        fs::rename(&self.0, &to)
            .map_err(|_| anyhow!("Failed to rename file {}", self.0.to_string_lossy()))
    }

    /// 元のファイル名を作成
    ///
    /// # Param
    /// - `delim`: Prefixと元のファイル名の区切り文字列
    pub fn create_reverted_name(&self, delim: &str) -> Option<String> {
        let org = self
            .0
            .file_name()
            .expect("File should have a name.")
            .to_string_lossy();

        match org.find(delim) {
            Some(prefix_index) => {
                let new_name = &org[prefix_index + 2..];
                Some(new_name.to_string())
            }
            None => None,
        }
    }

    /// ファイル名を元に戻す
    ///
    /// # Param
    /// - `delim`: Prefixと元のファイル名の区切り文字列
    pub fn revert_name(&self, delim: &str) -> Result<()> {
        let new_name = self
            .create_reverted_name(delim)
            .ok_or_else(|| anyhow!("Name delimiter is not found. {:?}", self.0))?;

        let parent = self.0.parent().expect("File should have a parent.");
        let mut to = PathBuf::from(parent);
        to.push(new_name);

        fs::rename(&self.0, &to)
            .map_err(|_| anyhow!("Failed to revert file {}", self.0.to_string_lossy()))
    }
}

impl From<PathBuf> for PhotoFile {
    fn from(path: PathBuf) -> Self {
        PhotoFile(path)
    }
}

impl Ord for PhotoFile {
    fn cmp(&self, other: &Self) -> Ordering {
        let f1 = fs::File::open(self.0.as_path());
        let f2 = fs::File::open(other.0.as_path());
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
                let time1 = e1.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY);
                let time2 = e2.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY);
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
}

impl PartialOrd for PhotoFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn create_prefix(num: usize, len: usize) -> String {
    let num = num.to_string();

    if len < num.len() {
        num
    } else {
        "0".repeat(len - num.len()) + num.as_str()
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
