#!/bin/bash

set -e

rm -rf build
rm -rf ~/rpmbuild

. build-scripts/common.sh

prepare_bundle
gen_manifest dnf

echo 'Packaging'
rpmdev-setuptree
if [[ $IS_HEADLESS = 0 ]]; then
    cp bundle/rpm/fig.spec ~/rpmbuild/SPECS/fig.spec
else
    cp bundle/rpm/fig-headless.spec ~/rpmbuild/SPECS/fig.spec
fi
sed -i "s/\$VERSION/${VERSION}/" ~/rpmbuild/SPECS/fig.spec
sed -i "s/\$ARCH/${ARCH}/" ~/rpmbuild/SPECS/fig.spec
mkdir -p ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
rm -r ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
cp -r build/ ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
rpmbuild -bb ~/rpmbuild/SPECS/fig.spec

if [[ $IS_HEADLESS = 0 ]]; then
    cp ~/rpmbuild/RPMS/${ARCH}/fig-${VERSION}-1.${ARCH}.rpm fig.rpm
else
    cp ~/rpmbuild/RPMS/${ARCH}/fig-headless-${VERSION}-1.${ARCH}.rpm fig.rpm
fi
