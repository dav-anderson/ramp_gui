linux:
	cargo build --target x86_64-unknown-linux-gnu

mac:
	cargo build --target aarch64-apple-darwin

dmg:
	rm -f dmg_contents/ramp_gui
	cp Users/davidanderson/ramp_gui/target/aarch64-apple-darwin/debug/ramp_gui Users/davidanderson/ramp_gui/dmg_contents
	hdiutil create -volname "Ramp Gui" -srcfolder dmg_contents -ov -format UDZO ramp_gui.dmg