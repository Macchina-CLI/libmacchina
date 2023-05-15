<div align="center">
<h1>libmacchina</h1>

A library providing access to all sorts of system information.

Linux • macOS • Windows • NetBSD • FreeBSD • Android • OpenWrt

<a href="https://crates.io/crates/libmacchina">
    <img src="https://img.shields.io/crates/v/libmacchina" alt="version" />
</a>

<a href="https://docs.rs/crate/libmacchina/">
    <img src="https://docs.rs/libmacchina/badge.svg" alt="docs" />
</a>

</div>

### Disclaimer

_libmacchina_ utilizes **unsafe** code in the form of calls to system libraries
that haven't been natively implemented in Rust, we do this for performance
reasons.

### Usage

Add the following to your project's _Cargo.toml_ file:

```toml
libmacchina = "7"
```

### Notes

On distributions like openSUSE that use the `ndb` RPM database format, `librpm`
(which is usually provided by the `rpm-devel` package) is required for the RPM
package count readout to work.

### Examples

```rust
// Let's import two of the several available types.
use libmacchina::{GeneralReadout, MemoryReadout};

fn main() {
    // Let's import the GeneralReadout trait so we
    // can fetch some general information about the host.
    use libmacchina::traits::GeneralReadout as _;

    let general_readout = GeneralReadout::new();

    // There are many more metrics we can query
    // i.e. username, distribution, terminal, shell, etc.
    let cpu_cores = general_readout.cpu_cores().unwrap(); // 8 [logical cores]
    let cpu = general_readout.cpu_model_name().unwrap();  // Intel(R) Core(TM) i5-8265U CPU @ 1.60GHz
    let uptime = general_readout.uptime().unwrap();       // 1500 [in seconds]

    // Now we'll import the MemoryReadout trait to get an
    // idea of what the host's memory usage looks like.
    use libmacchina::traits::MemoryReadout as _;

    let memory_readout = MemoryReadout::new();

    let total_mem = memory_readout.total(); // 20242204 [in kB]
    let used_mem = memory_readout.used();   // 3894880 [in kB]
}

```
