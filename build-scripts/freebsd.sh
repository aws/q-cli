# build
cargo build --release -p fig_cli
cargo build --release -p figterm

# archive
mkdir -p build/usr/bin

mv target/release/fig_cli build/usr/bin/fig
mv target/release/figterm build/usr/bin/figterm

tar -czf fig.tar.gz -C build .