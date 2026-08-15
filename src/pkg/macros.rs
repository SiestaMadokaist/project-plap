#[macro_export]
macro_rules! json_type {
    ($name:ident) => {
        impl TryFrom<serde_json::Value> for $name {
            type Error = serde_json::Error;
            fn try_from(value: serde_json::Value) -> Result<$name, Self::Error> {
                serde_json::from_value(value)
            }
        }
    };
}
macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> $name {
                $name(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }
    };
}

#[macro_export]
macro_rules! displayable {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
                write!(f, "{}", s.trim_matches('"'))
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.to_string()
            }
        }

        impl From<&$name> for String {
            fn from(value: &$name) -> String {
                value.to_string()
            }
        }
    };
}

macro_rules! trait_clients {
    ($name:ident, $($bound:path),+ $(,)?) => {
        pub trait $name: $($bound +)+ {}
        impl<T: $($bound +)+> $name for T {}
    };
}

macro_rules! trait_repos {
    ($name:ident, $($bound:path),+ $(,)?) => {
        pub trait $name: $($bound +)+ {}
        impl<T: $($bound +)+> $name for T {}
    };
}

pub use displayable;
pub(crate) use id_type;
pub use json_type;
pub(crate) use trait_clients;
pub(crate) use trait_repos;
