use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;
use nix::unistd::{Uid, Gid, chown};

use super::traits;

#[derive(Debug)]
pub struct Directory<'a> {
    pub path: &'a Path,
    pub mode: u32,
    pub owner: u32,
    pub group: u32,
}

impl<'a> traits::Step for Directory<'a> {
    fn apply(&self) -> Result<(), ()> {
        if self.path.is_file() {
            println!("{:?} is a file when we expected a directory, removing.", self.path);
            std::fs::remove_file(&self.path);
        } else if self.path.is_dir() {
            println!("{:?} already exists, just updating permissions", self.path);
        } else if !self.path.exists() {
            std::fs::create_dir_all(&self.path);
        }

        println!("{:?}", self.path.metadata().unwrap().permissions().mode());
        // Applying owner and user is idempotent - just run
        chown(self.path,
            Some(Uid::from_raw(self.owner)), Some(Gid::from_raw(self.group))).expect("failed to change ownership");
        let mut perms = self.path.metadata().unwrap().permissions();
        perms.set_mode(self.mode);
        std::fs::set_permissions(self.path, perms);
        Ok(())
    }
    fn dry_apply(&self) -> Result<(), ()> {
        Ok(())
    }
    fn delete(&self) -> Result<(), ()> {
        self.path.is_file();
        Ok(())
    }
}
