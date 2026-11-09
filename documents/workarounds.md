## Rust

## typeshare and proc_macro attribute bug
Optimal code:
```rust
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Copy, Clone)]
pub enum Response {
    Begins(ComputingResource),
}
```
Current workaround:
```rust
#[cfg(feature = "serde")]
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Response {
    Begins(ComputingResource),
}

#[cfg(not(feature = "serde"))]
#[derive(Debug, Clone)]
pub enum Response {
    Begins(ComputingResource),
}

```
