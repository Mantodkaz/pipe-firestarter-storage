# 🖥️ GUI

> ⚠️ **Windows Only**: This version is currently optimized for Windows only. For other platforms, please use CLI version.

> 🚧 **Work in Progress**: GUI version is still in development. Many features are not yet implemented and may have limited functionality compared to CLI version. Use CLI for full feature set.

## Prerequisites
To build, make sure you have [Rust](https://www.rust-lang.org/tools/install) for Windows installed.

## Enable GUI
```bash
cargo build --features gui --release
```
`pipe.exe` will be available in `./target/release/`

## Current Status
- ✅ Basic GUI structure and navigation
- ✅ Login/authentication panel
- ✅ Upload/download interface
- ✅ Local encryption/decryption
- ✅ Wallet management UI
- ✅ File listing and management

##