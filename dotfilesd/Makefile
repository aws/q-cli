all:
	cargo build --release

install: all
	sudo rm -f /usr/local/bin/dotfilesd
	sudo cp target/release/dotfilesd /usr/local/bin/dotfilesd

clean:
	cargo clean
