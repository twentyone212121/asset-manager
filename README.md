# Static Asset Manager for Rust

A compile-time static asset manager that embeds files directly into your Rust binary while providing a convenient API to access them.

## Features

- **Compile-time embedding**: all assets are embedded in the binary at compile time
- **Strong typing**: access assets via enum variants with IDE autocompletion
- **Filtering support**: include or exclude files using regular expressions
- **Zero runtime overhead**: No filesystem access or initialization required

## Usage

```rust
use asset_macros::assets;
use asset_traits::{Asset, AssetCollection};

assets!(UiAssets, "assets/ui", include: r"\.(png|jpg|svg)$");

let logo_bytes = UiAssets::LogoPng.bytes();
let logo_path = UiAssets::LogoPng.path();

for asset in UiAssets::all() {
    println!("UI asset: {} ({} bytes)", asset.path(), asset.bytes().len());
}
```

## Macro Options

The `assets!` macro accepts the following parameters:

```rust
assets!(
    EnumName,            // Name of the generated enum
    "path/to/assets",    // Directory containing assets
    include: r"regex",   // Optional regex for files to include
    ignore: r"regex"     // Optional regex for files to exclude
);
```

## When to Use

This crate is ideal for:

- Games and applications that need to bundle resources
- WebAssembly projects where filesystem access is limited
- Situations where you want to avoid runtime asset loading/initialization
- Projects where compile-time validation of assets is important

## When Not to Use

This crate is not suitable for:

- Very large assets (>10MB) that would bloat binary size
- Assets that change frequently during development (requires recompilation)
- Dynamic asset loading at runtime
- Applications where assets need to be updated without recompiling
