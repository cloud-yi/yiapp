use std::fmt;
use std::fmt::Display;
use log::Level;

pub use failure::{Backtrace, Context, Error, Fail, ResultExt};

pub type _YiResult<T> = Result<T, Error>;

pub type YiResult<T> = Result<T, YiError>;

pub type YiCli = Result<(), String>;
// pub type YiApp<T> = Result<T, Error>;

// pub type YiApi<T> = Result<T, YiApiError>;

#[macro_export]
macro_rules! yierr_print { () => (|e| { println!("{}", e); } ); }

#[macro_export]
macro_rules! yierr_debug { () => (|e| { println!("{}", (|| -> YiResult<()> { Err(e) })().yierr()); } ); }

#[macro_export]
macro_rules! yierr_info { () => (|e| { info!("{}", e); } ); }

#[macro_export]
macro_rules! yimap_err { () => (|e| { error!("{}", e); e} ); }

#[derive(Debug, Fail)]
pub enum YiErrorKind {
    #[fail(display = "{}", _0)]
    Info(String),

    #[fail(display = "{}", _0)]
    InfoStr(&'static str),

    #[fail(
        display = "shell color argument for --color must be auto, always, or \
                   never, but value: {}",
        _0
    )]
    ShellColor(String),

    #[fail(display = "")]
    Cli(i32),

    #[fail(display = "{}", _0)]
    Clap(clap::Error),

    #[fail(display = "{}", _0)]
    Opt(String),

    #[fail(display = "version {}", _0)]
    Ver(u32),

    #[fail(display = "io error")]
    StdIo,

    #[fail(display = "An unknown error kind has occurred.")]
    Unknown,
}

#[derive(Debug)]
pub struct YiError {
    inner: Context<YiErrorKind>,
}

impl Fail for YiError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for YiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner())
    }
}

impl YiError {
    #[inline]
    pub fn inner(&self) -> String {
        Self::_inner(&self.inner)
    }

    pub fn kind(&self) -> &YiErrorKind {
        self.inner.get_context()
    }

    fn _inner_print<T: Display + Fail>(thing: Option<T>, level: Level) {
        thing.map_or((), |t| {
            let cause = Self::_inner(&t);

            match level {
                Level::Info  => log::info!("{}{}", t, cause),
                Level::Warn  => log::warn!("{}{}", t, cause),
                Level::Debug => log::debug!("{}{}", t, cause),
                Level::Trace => log::trace!("{}{}", t, cause),
                _            => (),
            }
        })
    }

    fn error_cause<T: Display + Fail>(thing: Option<T>) -> String {
        thing.map_or(String::default(), |e| Self::_inner(&e))
    }

    fn _inner<T: Display + Fail>(e: &T) -> String {
        let mut cause = String::default();
        cause.push_str(&e.to_string());
        let mut err: &dyn Fail = e;
        while let Some(next) = err.cause() {
            cause.push_str(&format!(", {}", next.to_string()));
            err = next;
        }

        cause
    }
}

pub trait YiResultExt<T, E> {
    fn yierr(self) -> String;

    fn to_yierr<C: Display + Send + Sync>(self, text: C) -> YiResult<T>;

    fn to_yicli(self) -> YiResult<T>;
}

impl<T, E: Fail> YiResultExt<T, E> for Result<T, E> {
    fn yierr(self) -> String {
        YiError::error_cause(self.err())
    }

    fn to_yierr<C: Display + Send + Sync>(self, err: C) -> YiResult<T> {
        Ok(self.context(err.to_string())?)
    }

    fn to_yicli(self) -> YiResult<T> {
        Ok(self.context(YiErrorKind::Cli(101))?)
    }
}

#[inline]
pub fn to_yierr<T, E, C>(r: Result<T, E>, c: C) -> YiResult<T>
where C: Display + Send + Sync, E: Fail
{
    Ok(r.context(c.to_string())?)
}

#[inline]
pub fn with_yierr<T, E, C>(c: C) -> impl Fn(Result<T,E>) -> YiResult<T>
where C: Display + Send + Sync, E: Fail,
{
    move |r: Result<T,E>| -> YiResult<T> { Ok(r.context(c.to_string())?) }
}

#[inline]
pub fn yierr_str<E, C>(c: C) -> impl Fn(E) -> YiError
where C: Display + Send + Sync, E: Display,
{
    move |e: E| -> YiError {
        let mut str = e.to_string();
        if !str.is_empty() {
            str = format!(", inner cause: {}", str);
        }

        YiError::from(format!("{}{}", c, str))
    }
}

impl From<YiErrorKind> for YiError {
    fn from(kind: YiErrorKind) -> YiError {
        YiError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<YiErrorKind>> for YiError {
    fn from(inner: Context<YiErrorKind>) -> YiError {
        YiError { inner }
    }
}

impl From<Context<String>> for YiError {
    fn from(_e: Context<String>) -> YiError {
        let inner = _e.map(YiErrorKind::Info);
        YiError { inner }
    }
}

impl From<YiError> for String {
    fn from(yierr: YiError) -> String {
        yierr.inner()
    }
}

impl From<clap::Error> for YiError {
    fn from(_e: clap::Error) -> YiError {
        // let _ = if _e.use_stderr() { 1 } else { 0 };

        YiError::from(YiErrorKind::Clap(_e))
    }
}

impl From<std::io::Error> for YiError {
    fn from(_e: std::io::Error) -> YiError {
        YiError { inner: _e.context(YiErrorKind::StdIo) }
    }
}

impl From<String> for YiError {
    fn from(_e: String) -> YiError {
        YiError::from(YiErrorKind::Info(_e))
    }
}

impl From<Context<&'static str>> for YiError {
    fn from(_e: Context<&'static str>) -> YiError {
        let inner = _e.map(YiErrorKind::InfoStr);
        YiError { inner }
    }
}

impl From<&'static str> for YiError {
    fn from(_e: &'static str) -> YiError {
        YiError::from(YiErrorKind::InfoStr(_e))
    }
}

pub fn yierr<T>(_e: T) -> YiError
where
    T: Display,
{
    YiError::from(_e.to_string())
}

pub fn yierrkind<T>(_e: T) -> YiErrorKind
where
    T: Display,
    YiErrorKind: From<T>,
{
    YiErrorKind::from(_e)
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn yierr_from() -> crate::error::YiCli {
    //     Ok(())
    // }
}
