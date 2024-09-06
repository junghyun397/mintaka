use std::fmt::{Debug, Display};

#[macro_export]
macro_rules! impl_debug_by_display {
    ($name:ident) => {
        impl Debug for $name {

            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self, f)
            }

        }
    };
}
