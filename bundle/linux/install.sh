#!/bin/sh

# Install binaries to their location
mkdir -p /opt/fig/bin
find usr/bin -type f -exec install -Dm644 "{}" "/opt/fig/bin/{}" \;
find /opt/fig/bin -type f -exec ln -s "{}" "/usr/bin/{}" \;

# Install /usr/share files
find usr/share -type f -exec echo "{}" "/{}" \;

