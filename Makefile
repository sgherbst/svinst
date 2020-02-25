VERSION = $(patsubst "%",%, $(word 3, $(shell grep version Cargo.toml)))
BIN_NAME = svinst

.PHONY: clean release_lnx release_win release_mac

clean:
	cargo clean

release_lnx:
	cargo build --release --target=x86_64-unknown-linux-musl
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-lnx.zip target/x86_64-unknown-linux-musl/release/${BIN_NAME}

release_win:
	cargo build --release --target=x86_64-pc-windows-msvc
	7z a ${BIN_NAME}-v${VERSION}-x86_64-win.zip target/x86_64-pc-windows-msvc/release/${BIN_NAME}.exe

release_mac:
	cargo build --release --target=x86_64-apple-darwin
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-mac.zip target/x86_64-apple-darwin/release/${BIN_NAME}
