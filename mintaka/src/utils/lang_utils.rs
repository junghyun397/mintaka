#[macro_export] macro_rules! min {
    ($a:expr,$b:expr) => ({ if $a < $b { $a } else { $b } });
}

#[macro_export] macro_rules! max {
    ($a:expr,$b:expr) => ({ if $a > $b { $a } else { $b } });
}
