# DB Reader

A standalone tool to read and display the contents of a `redb` database used by the scanner to store encounter records.

## Build

### On any Linux (including Raspberry Pi)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# System dependencies (only gcc is required)
sudo apt install gcc

# Clone and build
git clone https://github.com/Thejulfi/telescrap-sr
cd telescrap-sr/crates/db-reader
cargo build --release
```

If you have this error : `error: linker `aarch64-unknown-linux-gnu-gcc` not found`, you need to set up the `aarch64-linux-gnu-gcc` in the cargo configuration file.

```bash
nano ~/telescrap-sr/crates/db-reader/.cargo/config.toml
```

Add:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Usage

Copy the `db-reader` binary where you want to run it:

```bash
cp target/release/db-reader /path/to/your/folder/
./db-reader /path/to/your/folder/database.redb
```

### Output example

```
Title                                        Date         Active     Club type     Resale link
------------------------------------------------------------------------------------------------------------------------
STADE ROCHELAIS / STADE FRANCAIS             2026-09-15   yes        StadeRochelais https://...
STADE ROCHELAIS / TOULOUSE                   2026-10-02   no         StadeRochelais https://...

Total: 2 record(s) (1 active, 1 inactive)
```
