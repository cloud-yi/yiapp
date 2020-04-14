use std::fmt;

type VerStr = &'static str;

pub struct Version {
    pub name: VerStr,
    pub major: VerStr,
    pub minor: VerStr,
    pub patch: VerStr,
    pub release: VerStr,
    pub pre_release: VerStr,

    pub channel: VerStr,
    pub short_hash: VerStr,
    pub hash: VerStr,
    pub date: VerStr,
}

impl Version {
    pub fn new() -> Self {
        Version {
            name: env!("CARGO_PKG_NAME"),
            major: env!("CARGO_PKG_VERSION_MAJOR"),
            minor: env!("CARGO_PKG_VERSION_MINOR"),
            patch: env!("CARGO_PKG_VERSION_PATCH"),
            release: env!("SRC_RELEASE"),
            pre_release: env!("CARGO_PKG_VERSION_PRE"),

            channel: env!("SRC_CHANNEL"),
            short_hash: env!("SRC_SHORT_HASH"),
            hash: env!("SRC_HASH"),
            date: env!("SRC_DATE"),
        }
    }

    pub fn full() -> String {
        let version = Version::new();

        let mut version_string = String::from(version.to_string());
        version_string.push_str("\n");

        let or_push = |field: &str, label: &str, out: &mut String| {
            if !field.is_empty() {
                out.push_str(&format!("{}: {}\n", label, field));
            }
        };

        or_push(version.release, "....release", &mut version_string);
        or_push(version.channel, "....channel", &mut version_string);
        or_push(version.hash,    "commit-hash", &mut version_string);
        or_push(version.date,    "commit-date", &mut version_string);

        version_string
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let if_empty = |str: &str, r| {
            if str.is_empty() { "".to_string() }  else { r }
        };

        write!(f, "{} {}{}{}", self.name, self.release,
               if_empty(self.channel, format!("-{}", self.channel)),
               if_empty(self.short_hash,
                        format!(" ({} {})", self.short_hash, self.date)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn ver() {
        assert!(false, format!("\n{}", Version::full()));
    }
}
