# build
cargo build --release -p cw_cli
cargo build --release -p figterm

# archive
mkdir -p build/usr/bin

mv target/release/cw_cli build/usr/bin/cw
mv target/release/figterm build/usr/bin/figterm

tar -czf fig.tar.gz -C build .