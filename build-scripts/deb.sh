#!/bin/bash

set -e

rm -rf build
rm -rf ~/rpmbuild

. build-scripts/common.sh

prepare_bundle
gen_manifest apt

echo 'Packaging'
mkdir -p build/DEBIAN
if [[ $IS_HEADLESS = 0 ]]; then
    cp bundle/deb/control build/DEBIAN/control
else
    cp bundle/deb/control_headless build/DEBIAN/control
fi
cp bundle/deb/prerm build/DEBIAN/prerm
sed -i "s/^Version:.*/Version: ${VERSION}/" build/DEBIAN/control
chmod 755 build/DEBIAN/prerm
dpkg-deb --build --root-owner-group -Zxz build fig.deb
