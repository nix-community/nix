// We have a script that generates a bunch of executable steps.
use std::path::Path;
use std::collections::HashSet;
use nix::unistd::{Uid, User};


trait Step: std::fmt::Debug {
    fn apply(&self) -> Result<(), ()>;
    fn dry_apply(&self) -> Result<(), ()>;
    fn delete(&self) -> Result<(), ()>;
}

#[derive(Debug)]
struct LocalFile<'a> {
    path: &'a Path,
    permissions: String,
    contents: String,
}

impl<'a> Step for LocalFile<'a> {
    fn apply(&self) -> Result<(), ()> {
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

#[derive(Debug)]
struct Users {
    n_users: u32,
    gid: u32,
    name: String,
}

impl Users {
    fn _delta(&self) -> Result<(HashSet<(Uid, String)>, HashSet<(Uid, String)>), ()> {
        let mut current = HashSet::new();
        let mut target = HashSet::new();
        let base = 30_000;

        for i in 1..self.n_users {
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

impl Step for Users {
   fn apply(&self) -> Result<(), ()> {
        let (remove, add) = self._delta()?;
        for x in remove {
            println!("remove {:?}", x);
        }
        for x in add {
            println!("add {:?}", x);
        }
        Ok(())
    }
    fn dry_apply(&self) -> Result<(), ()> {
        Ok(())
    }
    fn delete(&self) -> Result<(), ()> {
        Ok(())
    }
}

fn make_plan(crud: Vec<&dyn Step>) {
    for x in crud {
        println!("{:?}", x);
        x.apply();
    }
}

fn main() {
    let lf = LocalFile {
        path: &Path::new("/etc/nix/nix.conf"),
        permissions: "-rwxr--r--".to_string(),
        contents: include_str!("default-nix-conf").to_string(),
    };
    let lf_uid = Users {
        n_users: 16,
        gid: 30000,
        name: "nixbld".to_string(),
    };
    let desired: Vec<&dyn Step> = vec![
        &lf,
        &lf_uid,
    ];
    let plan = make_plan(desired);
    let gid = 30000;

}
