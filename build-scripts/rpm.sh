#!/bin/bash

set -e

rm -rf build
rm -rf ~/rpmbuild

. build-scripts/common.sh

prepare_bundle
KIND=rpm gen_manifest dnf

echo 'Packaging'
rpmdev-setuptree
if [[ $IS_MINIMAL = 0 ]]; then
    cp bundle/rpm/fig.spec ~/rpmbuild/SPECS/fig.spec
else
    cp bundle/rpm/fig-minimal.spec ~/rpmbuild/SPECS/fig.spec
fi
SPLIT=$(python3 build-scripts/rpm-ver.py $VERSION)
FIRST=$(echo "$SPLIT" | head -n 1)
SECOND=$(echo "$SPLIT" | tail -n 1)
sed -i "s/\$VERSION/${FIRST}/" ~/rpmbuild/SPECS/fig.spec
sed -i "s/\$RELEASE/${SECOND}/" ~/rpmbuild/SPECS/fig.spec
mkdir -p ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
rm -r ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
cp -r build/ ~/rpmbuild/BUILD/fig-${VERSION}-1.${ARCH}/
rpmbuild -bb --target "$ARCH" ~/rpmbuild/SPECS/fig.spec

if [[ $IS_MINIMAL = 0 ]]; then
    cp ~/rpmbuild/RPMS/${ARCH}/fig-${FIRST}-${SECOND}.${ARCH}.rpm fig.rpm
else
    cp ~/rpmbuild/RPMS/${ARCH}/fig-minimal-${FIRST}-${SECOND}.${ARCH}.rpm fig.rpm
fi
