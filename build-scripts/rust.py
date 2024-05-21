from functools import cache
from os import environ
import platform
import shutil
from typing import Dict, List
from util import info, isBrazil, isDarwin, isLinux, get_variant, isMusl, run_cmd_output, warn
from datetime import datetime, timezone


@cache
def build_hash() -> str:
    if environ.get("CODEBUILD_SOURCE_VERSION") is not None:
        build_hash = environ["CODEBUILD_SOURCE_VERSION"]
    else:
        try:
            build_hash = run_cmd_output(["git", "rev-parse", "HEAD"]).strip()
        except Exception as e:
            warn("Failed to get build hash:", e)
            build_hash = "unknown"
    info("build_hash =", build_hash)
    return build_hash


@cache
def build_time() -> str:
    build_time = datetime.now(timezone.utc).isoformat()
    info("build_time =", build_time)
    return build_time


@cache
def skip_fish_tests() -> bool:
    skip_fish_tests = shutil.which("fish") is None and isBrazil()
    if skip_fish_tests:
        warn("Skipping fish tests in brazil")
    return skip_fish_tests


@cache
def cargo_cmd_name() -> str:
    if isMusl():
        return "cross"
    else:
        return "cargo"


def rust_env(release: bool, linker=None) -> Dict[str, str]:
    env = {
        "CARGO_NET_GIT_FETCH_WITH_CLI": "true",
    }

    if release:
        rustflags = [
            "-C force-frame-pointers=yes",
        ]

        if linker:
            rustflags.append(f"-C link-arg=-fuse-ld={linker}")

        if isLinux():
            rustflags.append("-C link-arg=-Wl,--compress-debug-sections=zlib")

        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_PROFILE_RELEASE_LTO"] = "thin"
        env["RUSTFLAGS"] = " ".join(rustflags)

    if isDarwin():
        env["MACOSX_DEPLOYMENT_TARGET"] = "10.13"

    # TODO(grant): move Variant to be an arg of the functions
    env["AMAZON_Q_BUILD_TARGET_TRIPLE"] = get_target_triple()
    env["AMAZON_Q_BUILD_VARIANT"] = get_variant().name
    env["AMAZON_Q_BUILD_HASH"] = build_hash()
    env["AMAZON_Q_BUILD_DATETIME"] = build_time()
    env["Q_TELEMETRY_CLIENT_ID"] = "ffffffff-ffff-ffff-ffff-ffffffffffff"

    if skip_fish_tests():
        env["AMAZON_Q_BUILD_SKIP_FISH_TESTS"] = "1"

    return env


def rust_targets() -> List[str]:
    """
    Returns the supported rust targets for the current environment.
    """
    match platform.system():
        case "Darwin":
            return ["x86_64-apple-darwin", "aarch64-apple-darwin"]
        case "Linux":
            return [get_target_triple()]
        case other:
            raise ValueError(f"Unsupported platform {other}")


@cache
def get_target_triple() -> str:
    """
    Returns the target triple to be built and defined in the application manifest.
    """
    env = environ.get("AMAZON_Q_BUILD_TARGET_TRIPLE")
    if env:
        return env
    elif isDarwin():
        return "universal-apple-darwin"
    else:
        match platform.machine():
            case "x86_64":
                return "x86_64-unknown-linux-musl" if isMusl() else "x86_64-unknown-linux-gnu"
            case "aarch64":
                return "aarch64-unknown-linux-musl" if isMusl() else "aarch64-unknown-linux-gnu"
            case other:
                raise ValueError(f"Unsupported machine {other}")


if __name__ == "__main__":
    build_hash()
    build_time()
    info("rust_targets =", rust_targets())
