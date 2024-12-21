# bench-compress
`bench-compress` is a utility command to measure compression ratio and speed for arbitrary external programs.

It runs specified compression commands against a small portion of a provided file, and prints statistics about each command.

## Installation
You need to have a working Rust toolchain installed.
Run:
```shell
cargo install --git https://github.com/insomnimus/bench-compress --branch main
```

## Usage
```shell
# By default, only 3 commands are benchmarked (zstd, xz and xz with extreme settings)
# and only the first 50mb of the file is tried
bench-compress ./foo.tar

# To change the number of bytes being tried, specify -n
bench-compress -n 100mib ./foo.tar

# Use -n 0 to benchmark the whole file
bench-compress -n 0 ./foo.tar

# To provide your own commands to benchmark, provide them as arguments
bench-compress ./foo.tar "xz -4" "zstd -19"
```
