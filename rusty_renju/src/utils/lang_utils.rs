#[macro_export] macro_rules! min {
    ($a:expr,$b:expr) => ({ if $a < $b { $a } else { $b } });
}

#[macro_export] macro_rules! max {
    ($a:expr,$b:expr) => ({ if $a > $b { $a } else { $b } });
}

#[macro_export] macro_rules! boxed_slice {
    () => {
        Box::from([])
    };
    ($($elem:expr),+ $(,)?) => {
        Box::from([$($elem),*])
    };
}

#[macro_export] macro_rules! impl_debug_by_display {
    ($name:ident) => {
        impl Debug for $name {

            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self, f)
            }

        }
    };
}

pub const fn repeat_4x(source: u8) -> u32 {
    u32::from_ne_bytes([source, source, source, source])
}

pub const fn repeat_8x(source: u8) -> u64 {
    u64::from_ne_bytes([source, source, source, source, source, source, source, source])
}
