# lz77
implementation of the lz77 compression algorithm in rust

## File format
The file should be treated as groups of two bytes stored in big endian, eg: A0 F8 B3 DB EA FF would be:
(A0F8, B3DB, EAFF), where the first value is offset, second is match length, with the third being the next character

## Building and testing
### Prerequisites
Rust toolchain, can be found at https://www.rust-lang.org/tools/install
### Building
The project can be built with `cargo build`. This will generate a bianary in `./target/debug/project_name`

You can build and run it in the same command with `cargo run` followed by any arguments (EG: `cargo run compress ./tests/aviation.txt out.txt`)
### Testing
All performance testing should be done on bianaries built using the `--release` flag (EG: `cargo build --release` or `cargo run --release decompress foo bar`
