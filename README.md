<div align="center">
<h1>libmacchina</h1>

A library that can fetch all sorts of system information, super duper fast!

Linux • macOS • Windows • NetBSD • Android • OpenWrt

<a href="https://crates.io/crates/libmacchina">
    <img src="https://img.shields.io/crates/v/libmacchina" alt="version" />
</a>

<a href="https://docs.rs/crate/libmacchina/">
    <img src="https://docs.rs/libmacchina/badge.svg" alt="docs" />
</a>

</div>

### Usage

Add the following to your project's *Cargo.toml* file:

```toml
libmacchina = "1.0.5"
```

### Examples

```rust
// Let's import two of the several types.
use libmacchina::{GeneralReadout, MemoryReadout};

fn main() {
    // Let's import the GeneralReadout trait so we
    // can fetch some general information about the host.
    use libmacchina::traits::GeneralReadout as _;
    
    // There are too many  functions within GeneralReadout to list, but you get the gist ;)
    let general_readout = GeneralReadout::new();
    let cpu_cores = general_readout.cpu_cores().unwrap();          // 8
    let cpu = general_readout.cpu_model_name().unwrap();          // Intel(R) Core(TM) i5-8265U CPU @ 1.60GHz
    let uptime = general_readout.uptime().unwrap();                // 1500 [in seconds]

    // Now we'll import the MemoryReadout trait to get an
    // idea of what the host's memory usage looks like.
    use libmacchina::traits::MemoryReadout as _;

    let memory_readout = MemoryReadout::new();          
    let total_mem = memory_readout.total();       // 20242204 [in kB]
    let used_mem = memory_readout.used();         // 3894880 [in kB]
}

```
