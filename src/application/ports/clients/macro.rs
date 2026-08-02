macro_rules! impl_has {
    ($trait_name:ident, $assoc:ident, $method:ident, $container:ident) => {
        impl<CC: $container> $trait_name for CC {
            type $assoc = CC::$assoc;
            fn $method(&self) -> &Self::$assoc {
                $container::$method(self)
            }
        }
    };
}

pub(crate) use impl_has;
