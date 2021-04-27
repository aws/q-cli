 # (leading space required for Xenix /bin/sh)

#
# Determine the type of *ix operating system that we're
# running on, and echo an appropriate value.
# This script is intended to be used in Makefiles.
# (This is a kludge.  Gotta be a better way.)
#

case `uname -s` in
"FreeBSD")
	PLATFORM="freebsd"
	;;
"Linux")
	PLATFORM="linux"
	;;
"Darwin")
	PLATFORM="macos"
	;;
"SunOS")
	PLATFORM="solaris"
	;;
*)
	echo "Unknown platform" >&2
	exit 1
esac
echo $PLATFORM
exit 0
