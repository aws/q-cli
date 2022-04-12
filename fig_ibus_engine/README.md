## To develop
1. Make sure the IBus daemon is running
2. Start this crate
3. run `ibus engine FigIBusEngine`

## To install
1. Drop engine.xml (with proper target path) into `/usr/share/ibus/component`
2. Update the IBus cache (`ibus write-cache`)
3. Set the `FigIBusEngine` as the active engine in whatever configuration UI
