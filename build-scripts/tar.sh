#!/bin/bash

#!/bin/bash

set -e

rm -rf build
rm -f fig.tar.xz

. build-scripts/common.sh

prepare_bundle
KIND=tar gen_manifest unknown

echo 'Packaging'
cd build && tar cvfJ ../fig.tar.xz *
