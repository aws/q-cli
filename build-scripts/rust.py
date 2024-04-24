import platform
from typing import Dict, List
from util import isDarwin, isLinux, get_variant


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
    env["Q_BUILD_VARIANT"] = get_variant().name

    return env


def rust_targets() -> List[str]:
    match platform.system():
        case "Darwin":
            return ["x86_64-apple-darwin", "aarch64-apple-darwin"]
        case "Linux":
            match platform.machine():
                case "x86_64":
                    return ["x86_64-unknown-linux-gnu"]
                case "aarch64":
                    return ["aarch64-unknown-linux-gnu"]
                case other:
                    raise ValueError(f"Unsupported machine {other}")
        case other:
            raise ValueError(f"Unsupported platform {other}")
