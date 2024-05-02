use anyhow::{Context, Result};
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct RecursiveDirContents {
    pub dirs: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

pub fn get_home_dir() -> Result<PathBuf> {
    let home_dir = std::env::var("HOME").context("get user home directory from environment")?;
    Ok(PathBuf::from(home_dir))
}

pub fn walk_dir(path: &Path) -> Result<RecursiveDirContents> {
    let mut contents = RecursiveDirContents::default();
    let entries =
        fs::read_dir(path).with_context(|| format!("failed to read path: {}", path.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("failed to read path: {}", path.display()))?
            .path();
        if path.is_dir() {
            contents.dirs.push(path.clone());
            let mut subcontents = walk_dir(&path)?;
            contents.dirs.append(&mut subcontents.dirs);
            contents.files.append(&mut subcontents.files);
        } else {
            contents.files.push(path);
        }
    }
    Ok(contents)
}

pub fn get_relative_path<P: AsRef<Path>>(base: &Path, absolute_path: P) -> Result<PathBuf> {
    Ok(absolute_path
        .as_ref()
        .strip_prefix(base)
        .with_context(|| {
            format!(
                "{} is not relative to {}",
                absolute_path.as_ref().display(),
                base.display()
            )
        })?
        .to_path_buf())
}

pub fn copy_directory_full<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    let dir_contents = walk_dir(from.as_ref())?;
    for dir_path in dir_contents.dirs {
        let relative_path = get_relative_path(from.as_ref(), &dir_path)
            .with_context(|| format!("non-relative path: {}", dir_path.display()))?;
        let target_path = to.as_ref().to_path_buf().join(relative_path);
        std::fs::create_dir_all(&target_path).context("failed to create staging subdirectory")?;
    }
    for file_path in dir_contents.files {
        let relative_path = get_relative_path(from.as_ref(), &file_path)
            .with_context(|| format!("non-relative path: {}", file_path.display()))?;
        let target_path = to.as_ref().to_path_buf().join(relative_path);
        std::fs::copy(&file_path, &target_path)
            .with_context(|| format!("failed to copy to {}", target_path.display()))?;
        copy_file_mode(&file_path, &target_path)?;
    }
    Ok(())
}

pub fn copy_file_mode(src: &Path, dst: &Path) -> Result<()> {
    let metadata = fs::metadata(src)?;
    let permissions = metadata.permissions();
    fs::set_permissions(dst, Permissions::from_mode(permissions.mode()))
        .context("set permissions")?;
    Ok(())
}
