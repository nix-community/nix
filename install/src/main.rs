// We have a script that generates a bunch of executable steps.
use std::path::Path;

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

struct NixConf {
    system_features: String,
    timeout: u32,
}

fn make_plan(crud: Vec<&dyn Step>) {
    for x in crud {
        x.apply();
        println!("{:?}", x);
    }
}

fn main() {
    let lf = LocalFile {
        path: &Path::new("/etc/nix/nix.conf"),
        permissions: "-rwxr--r--".to_string(),
        contents: include_str!("default-nix-conf").to_string(),
    };
    let desired: Vec<&dyn Step> = vec![
        &lf
    ];
    let plan = make_plan(desired);
    // print(plan);
    // execute(plan);
}
