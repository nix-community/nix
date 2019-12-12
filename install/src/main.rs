// We have a script that generates a bunch of executable steps.
use clap::{App, Arg, SubCommand};
use std::path::Path;

mod traits;
mod users;
mod directory;

#[derive(Debug)]
struct LocalFile<'a> {
    permissions: String,
    path: &'a Path,
    contents: String,
}

impl<'a> traits::Step for LocalFile<'a> {
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

// TODO(tom): structurally it might make sense to have
// a nesting "Step" then the top level just becomes
// top_level.apply()
fn apply(steps: Vec<&dyn traits::Step>) {
    for x in steps {
        println!("apply {:?}", x);
        x.apply();
    }
}

fn delete(steps: Vec<&dyn traits::Step>) {
    for x in steps {
        println!("delete {:?}", x);
        x.delete();
    }
}

fn _check_correct_system() {
    // TODO(tom) - use whitelist instead of blacklist for installer
    if let Ok(s) = std::fs::read_to_string(&Path::new("/etc/os-release")) {
        if s.contains("nixos") {
            panic!("This looks like a nixos, aborting to avoid breakage");
        }
    }
}

fn _check_root() {
    if !nix::unistd::Uid::effective().is_root() {
        panic!("need to be root");
    };
}

fn main() {
    _check_correct_system();
    _check_root();

    let matches = App::new("Nix installer")
        .version("0.1")
        .about("Manage your local nix install")
        .subcommand(
            SubCommand::with_name("install")
                .about("Install or fix system")
                .arg(
                    Arg::with_name("num_build_users")
                        .long("num-build-users")
                        .help("How many build users to set up")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("uninstall").about("Completely removes nix from your system"),
        )
        .get_matches();

    // TODO(tom): design question - removal will remove all of the following but
    // if we don't do it recursively then we need to enumerate all subdirs..
    let lf = directory::Directory::root_owned("/nix/");
    let lf = directory::Directory::root_owned("/nix/var/log/nix/drvs");
    let lf = directory::Directory::root_owned("/nix/var/nix/db");
    let lf = directory::Directory::root_owned("/nix/var/nix/gcroots/per-user");
    let lf = directory::Directory::root_owned("/nix/var/nix/profiles");
    let lf = directory::Directory::root_owned("/nix/var/nix/temproots");
    let lf = directory::Directory::root_owned("/nix/var/nix/userpool/per-user");
    let lf = directory::Directory {
        path: &Path::new("/nix/store"),
        mode: 0x43fd,
        owner: 0,
        group: 30000,
    };
    let lf = directory::Directory {
        path: &Path::new("/etc/nix"),
        mode: 0x41ed,
        owner: 0,
        group: 0,
    };


    if let Some(matches) = matches.subcommand_matches("install") {
        let lf_uid = users::Users {
            n_users: matches
                .value_of("num_build_users")
                .unwrap_or("16")
                .parse()
                .unwrap(),
            gid: 30000,
            name: "nixbld".to_string(),
        };
        let desired: Vec<&dyn traits::Step> = vec![&lf];
        apply(desired);
    }

    if let Some(matches) = matches.subcommand_matches("uninstall") {
        let lf_uid = users::Users {
            n_users: 0,
            gid: 30000,
            name: "nixbld".to_string(),
        };
        let desired: Vec<&dyn traits::Step> = vec![&lf];
        delete(desired);
    }
}
