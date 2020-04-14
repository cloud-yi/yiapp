// #![allow(unused_imports)]
// #![allow(unused_doc_comments)]
// #![allow(unused_variables)]
// #![allow(dead_code)]
// #![allow(unused_mut)]

pub mod error;
pub mod arg;

pub use clap;

mod macros {
    #[macro_export] macro_rules! yiack {
        ($enum:ty, $num:ty, $arrays:expr) => {

            impl std::ops::Deref for $enum {
                type Target = $num;
                fn deref(&self) -> &$num {
                    $arrays.iter().find(|&(t, _)| self == t)
                        .map(|(_, c)| c)
                        .unwrap_or(&$arrays[$arrays.len()-1].1)
                }
            }

            impl std::convert::AsRef<$num> for $enum {
                fn as_ref(&self) -> &$num {
                    $arrays.iter().find(|&(t, _)| self == t)
                        .map(|(_, c)| c)
                        .unwrap_or(&$arrays[$arrays.len()-1].1)
                }
            }

            impl std::convert::AsRef<$enum> for $num {
                fn as_ref(&self) -> &$enum {
                    $arrays.iter().find(|&(_, c)| *self == *c)
                        .map(|(t, _)| t)
                        .unwrap_or(&$arrays[$arrays.len()-1].0)
                }
            }

        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    #[test]
    fn scratch() {
        (|| Ok(assert!(true, "on".parse::<bool>()?)) )()
            .err().map_or((), |e: str::ParseBoolError| println!("{}", e));
    }
}
