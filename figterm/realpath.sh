#!/bin/bash

realpath_replacement() {
  OURPWD=$PWD
  cd "$(dirname "$1")"
  LINK=$(readlink "$(basename "$1")")
  while [ "$LINK" ]; do
    cd "$(dirname "$LINK")"
    LINK=$(readlink "$(basename "$1")")
  done
  REALPATH="$PWD/$(basename "$1")"
  cd "$OURPWD"
  echo "$REALPATH"
}

case `uname -s` in
"Linux")
  REALPATH=$(realpath .)
  ;;
"Darwin")
  REALPATH=$(realpath_replacement ".")
  ;;
*)
  echo "Unknown platform" >&2
  exit 1
esac
echo $REALPATH
exit 0
