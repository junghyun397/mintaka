## Rust

### Wasm32 SIMD
Optimal Code:
```rust
#[inline(always)]
fn pattern_bitmask(patterns: u8x16, mask: u8) -> u16 {
    (patterns & Simd::splat(mask))
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}
```
Current workaround:
```rust
#[cfg_attr(target_arch = "wasm32", inline(never))]
#[cfg_attr(not(target_arch = "wasm32"), inline(always))]
fn pattern_bitmask(patterns: u8x16, mask: u8) -> u16 {
    (patterns & Simd::splat(mask))
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}
```

### ADT const params
Optimal code:
```rust
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum Color {
    #[default] Black = 0,
    White = 1,
}
```
Current workaround:
```rust
#![feature(adt_const_params)] // nightly feature

#[derive(std::marker::ConstParamTy, PartialEq, Eq, Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum Color {
    #[default] Black = 0,
    White = 1,
}
```

## typeshare and proc_macro attribute bug
Optimal code:
```rust
#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Copy, Clone)]
pub enum Response {
    Begins(ComputingResource),
}
```
Current workaround:
```rust
#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "CommandSchema"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone)]
pub enum Response {
    Begins(ComputingResource),
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum ResponseSchema {
        Begins(ComputingResource),
    }
}

```
