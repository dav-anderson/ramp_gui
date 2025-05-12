rm -f dmg_contents/ramp_gui

cp /Users/davidanderson/ramp_gui/target/aarch64-apple-darwin/debug/ramp_gui /Users/davidanderson/ramp_gui/dmg_contents

hdiutil create -volname "Ramp Gui" -srcfolder dmg_contents -ov -format UDZO ramp_gui.dmg

rm "/Volumes/NO NAME/ramp_gui.dmg"

cp /Users/davidanderson/ramp_gui/ramp_gui.dmg "/Volumes/NO NAME"

echo "updated dmg"