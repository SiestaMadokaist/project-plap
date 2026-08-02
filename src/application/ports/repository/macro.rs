macro_rules! impl_has {
    ($trait_name:ident, $assoc:ident, $method:ident, $container:ident) => {
        impl<RC: $container> $trait_name for RC {
            type $assoc = RC::$assoc;
            fn $method(&self) -> &Self::$assoc {
                $container::$method(self)
            }
        }
    };
}

pub(crate) use impl_has;
