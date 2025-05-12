#Create mac .dmg

first build your binary, at the moment this is done with `make mac` from the project root

next you will need to make sure the apple script exists, it is currnetly called `Install Ramp.app`

Next you will need to make sure `dmg_contents` contains 1. the latest binary, 2. your `Install Ramp.app` and 3. a readme 

Now run this to create your `.dmg`
```
hdiutil create -volname "Ramp Gui" \
               -srcfolder dmg_contents \
               -ov format UDZO \
               ramp_gui.dmg
```

export the dmg to your target machine and mount it with
`open ramp_gui.dmg`

now double click the install script


notes
backup script

```
xcode-select --install > /dev/null 2>&1
if [ 0 == $? ]; then
    sleep 1
    osascript <<EOD
tell application "System Events"
    tell process "Install Command Line Developer Tools"
        keystroke return
        click button "Agree" of window "License Agreement"
    end tell
end tell
EOD
else
    echo "Command Line Developer Tools are already installed!"
fi
```