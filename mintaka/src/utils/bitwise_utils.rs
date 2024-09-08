// accept unsigned non-zero integer
#[macro_export] macro_rules! pop_count_less_then_two_unchecked {
    ($source:expr) => ($source & ($source - 1) == 0);
}

// accept unsigned integer
#[macro_export] macro_rules! pop_count_less_then_two {
     ($source:expr) => ($source == 0 || $source & ($source - 1) == 0);
 }

// accept unsigned non-zero integer
#[macro_export] macro_rules! pop_count_less_then_three_unchecked {
     ($source:expr) => {{
         let temp = $source & ($source - 1);
         temp & (temp - 1) == 0
     }};
}

// accept unsigned non-zero integer
#[macro_export] macro_rules! pop_count_less_then_four_unchecked {
     ($source:expr) => {{
         let mut temp = $source & ($source - 1);
         temp &= (temp - 1);
         temp & (temp - 1) == 0
     }};
}
