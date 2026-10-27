//! Remembers the level being played between runs, in a one-line text file
//! in the usual per-user data folder:
//!
//! * macOS: `~/Library/Application Support/Rock/level.txt`
//! * Windows: `%APPDATA%\Rock\data\level.txt`
//! * Linux: `$XDG_DATA_HOME/rock/level.txt` (or `~/.local/share/rock/`)
//!
//! Saving is best effort: if the file can't be read or written the game
//! just starts at level 1.

use crate::game::LAST_LEVEL;
use std::path::{Path, PathBuf};

fn file() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "Rock").map(|d| d.data_dir().join("level.txt"))
}

/// The saved level, if there is a valid one.
pub fn load() -> Option<i32> {
    read(&file()?)
}

/// Remember `level` for next time.
pub fn store(level: i32) {
    if let Some(path) = file() {
        if let Err(e) = write(&path, level) {
            eprintln!("rock: can't save the level to {}: {e}", path.display());
        }
    }
}

fn read(path: &Path) -> Option<i32> {
    let n = std::fs::read_to_string(path).ok()?.trim().parse().ok()?;
    (1..=LAST_LEVEL).contains(&n).then_some(n)
}

fn write(path: &Path, level: i32) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, format!("{level}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_bad_files() {
        let dir = std::env::temp_dir().join(format!("rock-save-test-{}", std::process::id()));
        let path = dir.join("sub").join("level.txt");
        assert_eq!(read(&path), None);
        write(&path, 7).unwrap();
        assert_eq!(read(&path), Some(7));
        for bad in ["0", "10", "-1", "seven", ""] {
            std::fs::write(&path, bad).unwrap();
            assert_eq!(read(&path), None, "{bad:?}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
