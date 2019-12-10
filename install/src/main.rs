// We have a script that generates a bunch of executable steps.
use std::path::Path;

trait CRUD {
    fn read(self) -> Result<(), ()>;
    fn update(self) -> Result<(), ()>;
    fn delete(self) -> Result<(), ()>;
    fn create(self) -> Result<(), ()>;
}

#[derive(Debug)]
struct LocalFile<'a> {
    path: &'a Path,
    permissions: String,
    contents: String,
}

// recorded  local should exist   what do?
//        y      n            n   no-op
//        y      n            y   create
//        y      y            n   delete
//        y      y            y   no-op (unless different, in that case replace)
//        n      n            n   no-op
//        n      n            y   create
//        n      y            n   delete
//        n      y            y   no-op (unless different, in that case replace)


impl<'a> CRUD for LocalFile<'a> {
    fn read(self) -> Result<(), ()>{ Ok(())
    }
    fn update(self) -> Result<(), ()> { Ok(()) } // no-op
    fn create(self) -> Result<(), ()> {
        self.path.is_file();
        Ok(())
    }
    fn delete(self) -> Result<(), ()> {
        self.delete();
        Ok(())
    }
}


struct NixConf {
    system_features: String,
    timeout: u32,
}


fn main() {
    let desired = vec![
        &(LocalFile {
            path: &Path::new("/etc/nix/nix.conf"),
            permissions: "-rwxr--r--".to_string(),
            contents: include_str!("default-nix-conf").to_string(),
        })
    ];
    // let plan = make_plan(desired, recorded);
    // print(plan);
    // execute(plan);
}
