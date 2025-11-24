use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Writes contents to a file, creating parent directories if they don't exist.
///
/// # Arguments
///
/// * `path` - Path to the file to write.
/// * `contents` - Data to write to the file.
///
/// # Returns
///
/// `Ok(())` on success, or an `io::Error` on failure.
pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("subdir/test.txt");
        let content = "Hello, world!";

        write_file(&file_path, content).unwrap();

        let read_content = fs::read_to_string(file_path).unwrap();
        assert_eq!(read_content, content);
    }
}
