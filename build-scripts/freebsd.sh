# build
cargo build --release -p q_cli
cargo build --release -p figterm

# archive
mkdir -p build/usr/bin

mv target/release/q_cli build/usr/bin/q
mv target/release/figterm build/usr/bin/figterm

tar -czf fig.tar.gz -C build .