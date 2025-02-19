#[macro_export] macro_rules! const_for {
    ($idx:ident in $start:expr, $end:expr; $body:block) => {
        {
            let mut $idx = $start;
            while $idx < $end {
                $body
                $idx += 1;
            }
        }
    };
}

#[macro_export] macro_rules! assert_struct_sizes {
    ($t:ty, size=$size:expr, align=$align:expr) => {
        const _: () = assert!(std::mem::size_of::<$t>() == $size);
        const _: () = assert!(std::mem::align_of::<$t>() == $align);
    };
}

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

pub const fn repeat_16x(source: u8) -> u128 {
    u128::from_ne_bytes([source, source, source, source, source, source, source, source, source, source, source, source, source, source, source, source])
}
