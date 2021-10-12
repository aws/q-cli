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
