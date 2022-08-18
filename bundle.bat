::
:: build our binaries
::
cargo build --release || goto e

::
:: create fig_cli installer
::
candle -nologo -o target/wix/ fig_cli/fig_cli.wxs -ext WixBalExtension || goto e
light -nologo -o target/wix/fig_cli.msi target/wix/fig_cli.wixobj -ext WixBalExtension || goto e

::
:: create fig_desktop installer
::
candle -nologo -o target/wix/ fig_desktop/fig_desktop.wxs -ext WixBalExtension || goto e
light -nologo -o target/wix/fig_desktop.msi target/wix/fig_desktop.wixobj -ext WixBalExtension || goto e

::
:: create figterm installer
::
candle -nologo -o target/wix/ figterm/figterm.wxs -ext WixBalExtension || goto e
light -nologo -o target/wix/figterm.msi target/wix/figterm.wixobj -ext WixBalExtension || goto e

::
:: create bundle installer
::
candle -nologo -o target/wix/ fig.wxs -ext WixBalExtension || goto e
light -nologo -o target/wix/fig_installer.exe target/wix/fig.wixobj -ext WixBalExtension || goto e

exit 0

:e
echo failed to bundle
exit 1
