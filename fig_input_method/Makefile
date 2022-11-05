output_dir = build
app_name = FigInputMethod
bin_name = fig_input_method 

.PHONY: bundle
bundle:
	cargo build --release
	mkdir -p $(output_dir)/$(app_name).app/Contents
	mkdir -p $(output_dir)/$(app_name).app/Contents/MacOS
	mkdir -p $(output_dir)/$(app_name).app/Contents/Resources
	cp Info.plist $(output_dir)/$(app_name).app/Contents/
	pwd
	ditto ../target/release/$(bin_name) $(output_dir)/$(app_name).app/Contents/MacOS/
	cp -r resources/* $(output_dir)/$(app_name).app/Contents/Resources/

.PHONY: clean
clean:
	rm -rf $(output_dir)/$(app_name).app

.PHONY: run
run: bundle
	./$(output_dir)/$(app_name).app/Contents/MacOS/$(bin_name)


macos: $(BINARY_NAME)-darwin-universal
	install $(OUT_DIR)/$< $(OUT_DIR)/$(BINARY_NAME)

$(BINARY_NAME)-x86_64-apple-darwin:
	cargo build --target=x86_64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-aarch64-apple-darwin:
	cargo build --target=aarch64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-darwin-universal:
	cargo build --target=x86_64-apple-darwin --target=aarch64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$(BINARY_NAME)-x86_64-apple-darwin
	install $(TARGET_DIR)/aarch64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$(BINARY_NAME)-aarch64-apple-darwin
	lipo -create -output $(OUT_DIR)/$@ $(OUT_DIR)/$(BINARY_NAME)-x86_64-apple-darwin $(OUT_DIR)/$(BINARY_NAME)-aarch64-apple-darwin
