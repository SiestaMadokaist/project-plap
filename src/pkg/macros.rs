macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }
    };
}

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
    };
}

pub(crate) use displayable;
pub(crate) use id_type;
