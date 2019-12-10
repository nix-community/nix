use nix::unistd::{Uid, User};
use std::collections::HashSet;

use super::traits;

#[derive(Debug)]
pub struct Users {
    pub n_users: u32,
    pub gid: u32,
    pub name: String,
}

impl Users {
    fn _delta(&self, n_users: u32) -> Result<(HashSet<(Uid, String)>, HashSet<(Uid, String)>), ()> {
        let mut current = HashSet::new();
        let mut target = HashSet::new();
        let base = 30_000;

        for i in 1..n_users {
            target.insert((Uid::from_raw(base + i), format!("nixbld{}", i)));
        }

        // Ugly but assuming we own up to 2000 user accounts starting
        // at UID 30_000.
        for i in base..base + 2_000 {
            let res = User::from_uid(Uid::from_raw(i));
            if let Ok(Some(user)) = res {
                if user.name.starts_with("nixbld") {
                    current.insert((user.uid, user.name));
                }
            }
        }

        let remove = current.difference(&target).cloned().collect();
        let add = target.difference(&current).cloned().collect();

        Ok((remove, add))
   }
}

impl traits::Step for Users {
   fn apply(&self) -> Result<(), ()> {
        let (remove, add) = self._delta(self.n_users)?;
        for x in remove {
            println!("remove {:?}", x);
        }
        for x in add {
            println!("add {:?}", x);
        }
        Ok(())
    }
    fn dry_apply(&self) -> Result<(), ()> {
        let (remove, add) = self._delta(self.n_users)?;
        for x in remove {
            println!("remove {:?}", x);
        }
        for x in add {
            println!("add {:?}", x);
        }
        Ok(())
    }
    fn delete(&self) -> Result<(), ()> {
        // 0 users == remove all
        let (remove, add) = self._delta(0)?;
        for x in remove {
            println!("remove {:?}", x);
        }
        Ok(())
    }
}