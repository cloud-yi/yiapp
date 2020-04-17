use std::fmt::Display;
use std::ops::Deref;
use std::fs;
use std::hash::Hash;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::Deserialize;
use config::Config;

use failure::{Fail};
use super::error::{YiResult, YiResultExt};

const CLONE_SPAWN: &str = "__CLONE_SPAWN__";

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "loading from file")]
    File,

    #[fail(display = "loading from environment")]
    Env,

    #[fail(display = "failed to match command argument")]
    CmdArg,

    #[fail(display = "file to log")]
    LogFile,

    #[fail(display = "file to error")]
    ErrFile,
}

#[derive(Debug, PartialEq, Hash)]
pub enum Desc<'a> {
    // App Desciption
    Version(&'a str),
    Author(&'a str),
    About(&'a str),

    // Arg Desciption
    Short(&'a str),
    Long(&'a str),
    Long_,
    ValueName(&'a str),
    ValueName_,
    Help(&'a str),
    Default(&'a str),
    Required,
    Index(u64),
    Multiple,

    // Config Description
    File(&'a str),
    Env(&'a str),
}

pub type Descs<'a> = &'a [Desc<'a>];
pub type Opt<'a, T> = (T, Descs<'a>);
pub type Opts<'a, T> = &'a [Opt<'a, T>];

pub type Configs<T> = HashMap<T, Config>;

pub struct App<'a, T> {
    name: T,
    args: Config,
    clap: clap::App<'a, 'a>,
    config: Configs<T>,
    cdir: PathBuf
}

impl<'a, T> App<'a, T>
where T: Deref<Target=str> + AsRef<str> + Display + Hash + Eq + Clone,
{
    pub fn new(desc: Opt<'a, T>, opts: Opts<'a, T>) -> Self {
        let name = desc.0.clone();
        let args = Config::default();
        let clap = Self::clap(desc, opts);
        let config = HashMap::new();
        // FIXME: default workdir
        let cdir = env::current_dir().unwrap_or_else(|_| From::from("./"));

        App { name, args, clap, config, cdir }
    }

    pub fn args_into<'de, D: Deserialize<'de>>(&self) -> YiResult<D> {
        self.args.clone().try_into().to_yierr(Error::CmdArg)
    }

    pub fn config(mut self, opts: Opts<'a, T>, keys: &[&str]) -> YiResult<Self> {
        let matches = self.clap.clone().get_matches();

        for (k, descs) in opts {
            let mut c = Config::default();

            for desc in *descs {
                    match *desc {
                        Desc::File(path) => {
                            let mut cdir = self.cdir.clone();
                            let buf: PathBuf = From::from(path);
                            let buf = if buf.is_relative() {
                                cdir.push(path);
                                cdir.to_str()
                            } else {
                                None
                            };

                            c.merge(config::File::with_name(buf.unwrap_or(path)))
                                .to_yierr(Error::File)?;
                        }

                        Desc::Env(env) => {
                            c.merge(config::Environment::with_prefix(env))
                                .to_yierr(Error::Env)?;
                        }

                        _ => (),
                    }
            }

            if k == &self.name {
                Self::arg_matches(keys, &matches, &mut c)?;
                self.args = c;
            } else {
                self.config.insert(k.clone(), c);
            }
        }

        self.spwan()?;

        Ok(self)
    }

    pub fn spwan(&self) -> YiResult<()> {
        let spawn: bool = self.get_arg("spawn").unwrap_or(false);

        if env::var(CLONE_SPAWN).ok().is_none() && spawn {
            if let Some(exe) = env::current_exe()?.to_str() {
                use std::process::{ Command, Stdio };

                let mut log_file = fs::OpenOptions::new();
                let mut err_file = fs::OpenOptions::new();

                let path = |ext| {
                    let name = format!("{}/{}{}", "log", &*self.name, ext);
                    self.filepath(&name)
                };

                let log_path = path(".log");
                // let err_path = path(".err");
                let err_path = path(".log"); // FIXME: error information 
                let pid_path = path(".pid");

                #[cfg(debug_assertions)]
                println!("logfile: {:?}, {:?}, {:?}", log_path, err_path, pid_path);

                if !log_path.exists() {
                    log_file.create(true);
                }
                if !err_path.exists() {
                    err_file.create(true);
                }

                log_file.write(true).append(true);
                err_file.write(true).append(true);

                let log_file = log_file.open(log_path).to_yierr(Error::LogFile)?;
                let err_file = err_file.open(err_path).to_yierr(Error::ErrFile)?;

                let mut _child = Command::new(exe)
                    .env(CLONE_SPAWN, "")
                    .args(env::args().skip(1))
                    .stdin(Stdio::null())
                    .stdout(log_file)
                    .stderr(err_file)
                    .spawn()?;

                #[cfg(windows)]
                _child.wait()?;

                #[cfg(debug_assertions)]
                println!("parent process exit");

                std::process::exit(0);
            }
        }

        Ok(())
    }

    pub fn filepath(&self, name: &str) -> PathBuf {
        if &name[0..1] == "/" {
            PathBuf::from(name)
        } else {
            let mut p = self.cdir.clone();
            p.push(name);
            p
        }
    }

    pub fn get_args(&self) -> &Config {
        &self.args
    }

    pub fn get_arg<'de, D: Deserialize<'de>>(&self, key: &'de str) -> YiResult<D> {
        self.args.get(key).to_yierr(Error::CmdArg)
    }

    pub fn with_subclap(mut self, subs: &[clap::App<'a, 'a>]) -> Self {
        let apps = self.clap.subcommands(subs.to_vec());
        self.clap = apps;
        self
    }

    pub fn clap(desc: Opt<'a, T>, args: Opts<'a, T>) -> clap::App<'a, 'a> {
        Self::inner_clap(desc, args, false)
    }

    pub fn sub_clap(desc: Opt<'a, T>, args: Opts<'a, T>) -> clap::App<'a, 'a> {
        Self::inner_clap(desc, args, true)
    }

    fn inner_clap(desc: Opt<'a, T>, args: Opts<'a, T>, subcmd: bool)
                  -> clap::App<'a, 'a> {
        let app = if subcmd {
            clap::SubCommand::with_name(desc.0.as_ref())
        } else {
            clap::App::new(desc.0.as_ref())
        };

        let app = desc.1.iter().fold(app, |app, desc| {
            match desc {
                Desc::About(v) => app.about(*v),
                Desc::Author(v) => app.author(*v),
                Desc::Version(v) => app.version(*v),
                _             => app,
            }
        });

        args.iter().fold(app, |app, (k, opts)| {
            let name = k.as_ref();
            let arg = clap::Arg::with_name(name);
            let arg = opts.iter().fold(arg, |arg, desc| {
                match desc {
                    Desc::Index(v) => arg.index(*v),
                    Desc::Help(v) => arg.help(*v),
                    Desc::Short(v) => arg.short(*v),
                    Desc::Long(v) => arg.long(*v),
                    Desc::Long_   => arg.long(name),
                    Desc::ValueName(v) => arg.value_name(*v),
                    Desc::ValueName_ => arg.value_name(name),
                    Desc::Multiple => arg.multiple(true),
                    Desc::Required => arg.required(true),
                    Desc::Default(v) => arg.default_value(*v),
                    _          => arg,
                }
            });

            app.arg(arg)
        })
    }

    fn arg_matches(keys: &[&str], matches: &clap::ArgMatches<'a>, config: &mut Config)
                   -> YiResult<()> {

        let mut prefix = "";

        for k in keys {
            if k.ends_with('.') {
                prefix = k;
                continue;
            }
            let ck = format!("{}{}", prefix, k);

            let v = matches.value_of(k);
            if matches.occurrences_of(k) > 0 {
                if let Some(v) = v {
                    config.set(&ck, v).to_yierr(Error::CmdArg)?;
                } else {
                    config.set(&ck, true).to_yierr(Error::CmdArg)?;
                }
            } else if config.get_str(&ck).is_err() {
                if let Some(v) = v {
                    config.set(&ck, v).to_yierr(Error::CmdArg)?;
                } else {
                    config.set(&ck, false).to_yierr(Error::CmdArg)?;
                }
            }
        }

        #[cfg(debug_assertions)]
        println!("config: {:?}", config);

        Ok(())
    }

}

mod macros {
    #[macro_export] macro_rules! yiarg {
        ($enum:ty, $strs:expr) => {
            impl std::fmt::Display for $enum {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }

            impl $enum {
                pub fn as_str(&self) -> &str {
                    let name = self.to_string().to_lowercase();
                    $strs.iter()
                        .find(|&&k| name.starts_with(k))
                        .map_or("noname", |k| k)
                }

                pub fn as_key(&self) -> String {
                    let name = self.to_string().to_lowercase();
                    let mut key = String::new();
                    for k in $strs {
                        if k.ends_with(".") {
                            key.clear();
                            key.push_str(k);
                            continue;
                        } else if name.starts_with(k) {
                            key.push_str(k);
                            break;
                        }
                    }
                    key
                }
            }

            impl std::convert::AsRef<str> for $enum {
                fn as_ref(&self) -> &str {
                    self.as_str()
                }
            }

            impl std::ops::Deref for $enum {
                type Target = str;

                fn deref(&self) -> &str {
                    self.as_str()
                }
            }
        };
    }
}
