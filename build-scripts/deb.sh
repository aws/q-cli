#!/bin/bash

set -e

rm -rf build
rm -rf ~/rpmbuild

. build-scripts/common.sh

prepare_bundle
KIND=deb gen_manifest apt

case $ARCH in
    x86_64)
        APT_ARCH=amd64;;
    aarch64)
        APT_ARCH=arm64;;
    *)
        echo AAAAAAA
        exit 1
    ;;
esac

echo 'Packaging'
mkdir -p build/DEBIAN
if [[ $IS_MINIMAL = 0 ]]; then
    cat bundle/deb/control | APT_ARCH=$APT_ARCH envsubst > build/DEBIAN/control
else
    cat bundle/deb/control_minimal | APT_ARCH=$APT_ARCH envsubst > build/DEBIAN/control
fi
cp bundle/deb/prerm build/DEBIAN/prerm
if [[ $IS_MINIMAL = 0 ]]; then
    cp bundle/deb/postrm build/DEBIAN/postrm
    chmod 755 build/DEBIAN/postrm
else
    printf '#!/bin/bash\n# dpkg crashes if we upgrade from a bad postrm to a package without a postrm' > build/DEBIAN/postrm
    chmod 755 build/DEBIAN/postrm
fi
cp bundle/deb/postinst build/DEBIAN/postinst
sed -i "s/^Version:.*/Version: ${VERSION}/" build/DEBIAN/control
chmod 755 build/DEBIAN/prerm
chmod 755 build/DEBIAN/postinst
dpkg-deb --build --root-owner-group -Zxz build fig.deb
