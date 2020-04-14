use std::env;
use std::path::Path;
use std::process::Command;

fn main() {

    // Check .git/HEAD dirty status
    if Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }

    let ci = Commit;

    println!("cargo:rerun-if-env-changed=CFG_RELEASE_CHANNEL");
    println!("cargo:rustc-env=SRC_RELEASE={}", ci.release());
    println!("cargo:rustc-env=SRC_CHANNEL={}", ci.channel());
    println!("cargo:rustc-env=SRC_HASH={}", ci.hash());
    println!("cargo:rustc-env=SRC_SHORT_HASH={}", ci.short_hash());
    println!("cargo:rustc-env=SRC_DATE={}", ci.date());

    // let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // File::create(out_dir.join("commit-info.txt"))
    //     .unwrap()
    //     .write_all(Commit::info().as_bytes())
    //     .unwrap();
}

struct Commit;

impl Commit {
    // Get hash and date of the last commit.
    // If wrong (not git installed or not a git repository) then return an empty string.
    // fn ver(&self) -> String {
    //     format!("{} {}, {}",
    //             env!("CARGO_PKG_NAME"),
    //             self.release(),
    //             self.info())
    // }

    fn release(&self) -> String {
        format!("{}.{}.{}",
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
                env!("CARGO_PKG_VERSION_PATCH"))
    }

    // fn info(&self) -> String {
    //     match (self.channel(), self.hash(), self.date()) {
    //         (channel, Some(hash), Some(date)) => {
    //             format!("{} ({} {})", channel, hash.trim_right(), date)
    //         }
    //         _ => String::new(),
    //     }
    // }

    fn channel(&self) -> String {
        if let Ok(channel) = env::var("CFG_RELEASE_CHANNEL") {
            channel
        } else {
            "nightly".to_owned()
        }
    }

    fn hash(&self) -> String {
        Self::exec_git(&["rev-parse", "HEAD"])
    }

    fn short_hash(&self) -> String {
        Self::exec_git(&["rev-parse", "--short", "HEAD"])
    }

    fn date(&self) -> String {
        Self::exec_git(&["log", "-1", "--date=short", "--pretty=format:%cd"])
    }

    fn exec_git(args: &[&str]) -> String {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .and_then(|r| String::from_utf8(r.stdout).ok())
            .map_or("".to_string(), |r| r)
    }
}
