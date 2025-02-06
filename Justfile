set positional-arguments 

build-aarch64:
  cargo build --target=aarch64-unknown-linux-gnu

# This assumes debian package paths
run-aarch64 *args:
  LD_LIBRARY_PATH=/usr/aarch64-linux-gnu/lib/ qemu-aarch64 -cpu cortex-a76 /usr/aarch64-linux-gnu/lib/ld-linux-aarch64.so.1 ./target/aarch64-unknown-linux-gnu/debug/jitcalc $@
