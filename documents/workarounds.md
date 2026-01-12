## ADT const params
Optimal code:
```rust
```

## Portable SIMD

## typeshare and proc_macro attribute bug
Optimal code:
```rust
#[typeshare::typeshare]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", content = "content"),
)]
#[derive(Debug, Copy, Clone)]
pub enum Response {
    Begins(ComputingResource),
}
```
Current workaround:
```rust
#[typeshare(serialized_as = "CommandSchema")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum Response {
    Begins(ComputingResource),
}

#[cfg(any())]
mod typeshare_workarounds {
    use super::*;
    #[cfg(feature = "serde")]
    #[typeshare::typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum ResponseSchema {
        Begins(ComputingResource),
    }
}

```
