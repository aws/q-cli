#!/bin/sh

# Install binaries to their location
mkdir -p /opt/fig/bin
find usr/bin -type f -exec install -Dm755 "{}" "/opt/fig/bin/{}" \;
find /opt/fig/bin -type f -exec ln -s "{}" "/usr/bin/{}" \;

# Install other /usr files
find usr/share -type f -exec install -Dm644 "{}" "/{}" \;
find usr/lib -type f -exec install -Dm644 "{}" "/{}" \;

