output_dir = build
app_name = FigInputMethod
bin_name = fig_input_method 

.PHONY: app
app:
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
run:
	./$(output_dir)/$(app_name).app/Contents/MacOS/$(bin_name)
