from enum import Enum
import os
import json
import datetime
import pathlib
import platform
import shutil
import sys
import itertools
from typing import Dict, List, Mapping, Sequence
from util import isDarwin, isLinux, run_cmd, run_cmd_output, info
from signing import (
    EcSigningData,
    EcSigningType,
    load_gpg_signer,
    rebundle_dmg,
    ec_sign_file,
    apple_notarize_file,
)
from importlib import import_module

APP_NAME = "CodeWhisperer"
BUILD_DIR_RELATIVE = pathlib.Path(os.environ.get("BUILD_DIR") or "build")
BUILD_DIR = BUILD_DIR_RELATIVE.absolute()


class Variant(Enum):
    FULL = 1
    MINIMAL = 2


def get_variant() -> Variant:
    match platform.system():
        case "Darwin":
            return Variant.FULL
        case "Linux":
            return Variant.MINIMAL
        case other:
            raise ValueError(f"Unsupported platform {other}")


def build_npm_packages() -> Dict[str, pathlib.Path]:
    run_cmd(["pnpm", "install", "--frozen-lockfile"])
    run_cmd(["pnpm", "build"])

    # copy to output
    dashboard_path = BUILD_DIR / "dashboard"
    shutil.rmtree(dashboard_path, ignore_errors=True)
    shutil.copytree("apps/dashboard/dist", dashboard_path)

    autocomplete_path = BUILD_DIR / "autocomplete"
    shutil.rmtree(autocomplete_path, ignore_errors=True)
    shutil.copytree("apps/autocomplete/dist", autocomplete_path)

    return {"dashboard": dashboard_path, "autocomplete": autocomplete_path}


def rust_env(linker=None) -> Dict[str, str]:
    rustflags = [
        "-C force-frame-pointers=yes",
    ]

    if linker:
        rustflags.append(f"-C link-arg=-fuse-ld={linker}")

    if isLinux():
        rustflags.append("-C link-arg=-Wl,--compress-debug-sections=zlib")

    env = {
        "CARGO_INCREMENTAL": "0",
        "CARGO_PROFILE_RELEASE_LTO": "thin",
        "RUSTFLAGS": " ".join(rustflags),
        "CARGO_NET_GIT_FETCH_WITH_CLI": "true",
    }

    if isDarwin():
        env["MACOSX_DEPLOYMENT_TARGET"] = "10.13"

    # TODO(grant): move Variant to be an arg of the functions
    env["CW_BUILD_VARIANT"] = get_variant().name

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


def build_cargo_bin(
    package: str,
    output_name: str | None = None,
    features: Mapping[str, Sequence[str]] | None = None,
) -> pathlib.Path:
    args = ["cargo", "build", "--release", "--locked", "--package", package]

    targets = rust_targets()
    for target in targets:
        args.extend(["--target", target])

    if features and features.get(package):
        args.extend(["--features", ",".join(features[package])])

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(),
        },
    )

    # create "universal" binary for macos
    if isDarwin():
        out_path = BUILD_DIR / f"{output_name or package}-universal-apple-darwin"

        args = [
            "lipo",
            "-create",
            "-output",
            out_path,
        ]
        for target in targets:
            args.extend(
                [
                    f"target/{target}/release/{package}",
                ]
            )
        run_cmd(args)
        return out_path
    else:
        # linux does not cross compile arch
        target = targets[0]
        target_path = pathlib.Path("target") / target / "release" / package
        out_path = BUILD_DIR / "bin" / (output_name or package)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(target_path, out_path)
        return out_path


def run_cargo_tests(features: Mapping[str, Sequence[str]] | None = None):
    # build the tests using the same config as normal builds
    args = ["cargo", "build", "--tests", "--release", "--locked"]

    for target in rust_targets():
        args.extend(["--target", target])

    if isLinux():
        args.extend(["--workspace", "--exclude", "fig_desktop"])

    if features:
        args.extend(
            [
                "--features",
                ",".join(set(itertools.chain.from_iterable(features.values()))),
            ]
        )

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(),
        },
    )

    args = ["cargo", "test", "--release", "--locked"]

    # disable fig_desktop tests for now
    if isLinux():
        args.extend(["--workspace", "--exclude", "fig_desktop"])

    if features:
        args.extend(
            [
                "--features",
                ",".join(set(itertools.chain.from_iterable(features.values()))),
            ]
        )

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(),
        },
    )


def run_clippy(features: Mapping[str, Sequence[str]] | None = None):
    # run clippy using the same config as normal builds
    args = ["cargo", "clippy", "--release", "--locked"]

    for target in rust_targets():
        args.extend(["--target", target])

    if isLinux():
        args.extend(["--workspace", "--exclude", "fig_desktop"])

    if features:
        args.extend(
            [
                "--features",
                ",".join(set(itertools.chain.from_iterable(features.values()))),
            ]
        )

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(),
        },
    )


def version() -> str:
    output = run_cmd_output(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
        ]
    )
    data = json.loads(output)
    for pkg in data["packages"]:
        if pkg["name"] == "fig_desktop":
            return pkg["version"]
    raise ValueError("Version not found")


def gen_manifest() -> str:
    variant = "full"

    dc_output = run_cmd_output(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
        ]
    )
    dc = json.loads(dc_output)["metadata"]["channel"]

    return json.dumps(
        {
            "managed_by": "dmg",
            "packaged_at": datetime.datetime.now().isoformat(),
            "packaged_by": "amazon",
            "variant": variant,
            "version": version(),
            "kind": "dmg",
            "default_channel": dc,
        }
    )


def build_macos_ime(signing_data: EcSigningData | None) -> pathlib.Path:
    fig_input_method_bin = build_cargo_bin("fig_input_method")
    input_method_app = pathlib.Path("build/CodeWhispererInputMethod.app")

    (input_method_app / "Contents/MacOS").mkdir(parents=True, exist_ok=True)

    shutil.copy2(
        fig_input_method_bin,
        input_method_app / "Contents/MacOS/fig_input_method",
    )
    shutil.copy2(
        "fig_input_method/Info.plist",
        input_method_app / "Contents/Info.plist",
    )
    shutil.copytree(
        "fig_input_method/resources",
        input_method_app / "Contents/Resources",
        dirs_exist_ok=True,
    )

    if signing_data:
        info("Signing macos ime")
        ec_sign_file(input_method_app, EcSigningType.IME, signing_data)
        apple_notarize_file(input_method_app, signing_data)

    return input_method_app


def tauri_config(cw_cli_path: pathlib.Path, cwterm_path: pathlib.Path, target: str) -> str:
    config = {
        "tauri": {
            "bundle": {
                "externalBin": [
                    str(cw_cli_path).removesuffix(f"-{target}"),
                    str(cwterm_path).removesuffix(f"-{target}"),
                ],
                "resources": ["manifest.json"],
            }
        }
    }
    return json.dumps(config)


def build_desktop_app(
    cwterm_path: pathlib.Path,
    cw_cli_path: pathlib.Path,
    npm_packages: Dict[str, pathlib.Path],
    signing_data: EcSigningData | None,
    features: Mapping[str, Sequence[str]] | None = None,
) -> pathlib.Path:
    target = "universal-apple-darwin"

    info("Building macos ime")
    ime_app = build_macos_ime(signing_data)

    info("Writing manifest")
    manifest_path = pathlib.Path("fig_desktop/manifest.json")
    manifest_path.write_text(gen_manifest())

    info("Building tauri config")
    tauri_config_path = pathlib.Path("fig_desktop/build-config.json")
    tauri_config_path.write_text(tauri_config(cw_cli_path=cw_cli_path, cwterm_path=cwterm_path, target=target))

    info("Building fig_desktop")

    cargo_tauri_args = [
        "cargo-tauri",
        "build",
        "--config",
        "build-config.json",
        "--target",
        target,
    ]

    if features and features.get("fig_desktop"):
        cargo_tauri_args.extend(["--features", ",".join(features["fig_desktop"])])

    run_cmd(
        cargo_tauri_args,
        cwd="fig_desktop",
        env={**os.environ, **rust_env(), "BUILD_DIR": BUILD_DIR},
    )

    # clean up
    manifest_path.unlink(missing_ok=True)
    tauri_config_path.unlink(missing_ok=True)

    target_bundle = pathlib.Path(f"target/{target}/release/bundle/macos/codewhisperer_desktop.app")
    app_path = BUILD_DIR / "CodeWhisperer.app"
    shutil.rmtree(app_path, ignore_errors=True)
    shutil.copytree(target_bundle, app_path)

    info_plist_path = app_path / "Contents/Info.plist"

    # Change the display name of the app
    run_cmd(
        [
            "defaults",
            "write",
            info_plist_path,
            "CFBundleDisplayName",
            APP_NAME,
        ]
    )
    run_cmd(
        [
            "defaults",
            "write",
            info_plist_path,
            "CFBundleName",
            APP_NAME,
        ]
    )

    # Specifies the app is an "agent app"
    run_cmd(["defaults", "write", info_plist_path, "LSUIElement", "-bool", "TRUE"])

    # Add codewhisperer:// association to bundle
    run_cmd(
        [
            "plutil",
            "-insert",
            "CFBundleURLTypes",
            "-xml",
            """<array>
    <dict>
        <key>CFBundleURLName</key>
        <string>com.amazon.codewhisperer</string>
        <key>CFBundleURLSchemes</key>
        <array>
        <string>codewhisperer</string>
        </array>
    </dict>
    </array>
    """,
            info_plist_path,
        ]
    )

    info("Copying CodeWhispererInputMethod.app into bundle")
    helpers_dir = app_path / "Contents/Helpers"
    helpers_dir.mkdir(parents=True, exist_ok=True)
    shutil.copytree(ime_app, helpers_dir.joinpath("CodeWhispererInputMethod.app"))

    info("Grabbing themes")
    theme_repo = BUILD_DIR / "themes"
    shutil.rmtree(theme_repo, ignore_errors=True)
    run_cmd(["git", "clone", "https://github.com/withfig/themes.git", theme_repo])
    shutil.copytree(theme_repo / "themes", app_path / "Contents/Resources/themes")

    for package, path in npm_packages.items():
        info(f"Copying {package} into bundle")
        shutil.copytree(path, app_path / "Contents/Resources" / package)

    dmg_path = BUILD_DIR / "CodeWhisperer.dmg"
    dmg_path.unlink(missing_ok=True)

    dmg_resources_dir = pathlib.Path("bundle/dmg")
    background_path = dmg_resources_dir / "background.png"
    icon_path = dmg_resources_dir / "VolumeIcon.icns"

    # we use a dynamic import here so that we dont use this dep
    # on other platforms
    dmgbuild = import_module("dmgbuild")

    dmgbuild.build_dmg(
        volume_name=APP_NAME,
        filename=dmg_path,
        settings={
            "format": "ULFO",
            "background": str(background_path),
            "icon": str(icon_path),
            "text_size": 12,
            "icon_size": 160,
            "window_rect": ((100, 100), (660, 400)),
            "files": [str(app_path)],
            "symlinks": {"Applications": "/Applications"},
            "icon_locations": {
                app_path.name: (180, 170),
                "Applications": (480, 170),
            },
        },
    )

    info(f"Created dmg at {dmg_path}")

    if signing_data:
        sign_and_rebundle_macos(app_path=app_path, dmg_path=dmg_path, signing_data=signing_data)

    return dmg_path


def sign_and_rebundle_macos(app_path: pathlib.Path, dmg_path: pathlib.Path, signing_data: EcSigningData):
    info("Signing app and dmg")

    # Sign the application
    ec_sign_file(app_path, EcSigningType.APP, signing_data)

    # Notarize the application
    apple_notarize_file(app_path, signing_data)

    # Rebundle the dmg file with the signed and notarized application
    rebundle_dmg(app_path=app_path, dmg_path=dmg_path)

    # Sign the dmg
    ec_sign_file(dmg_path, EcSigningType.DMG, signing_data)

    # Notarize the dmg
    apple_notarize_file(dmg_path, signing_data)

    info("Done signing!!")


def linux_bundle(
    cwterm_path: pathlib.Path,
    cw_cli_path: pathlib.Path,
    codewhisperer_desktop_path: pathlib.Path,
    is_minimal: bool,
):
    if not is_minimal:
        for res in [16, 22, 24, 32, 48, 64, 128, 256, 512]:
            shutil.copy2(
                f"fig_desktop/icons/{res}x{res}.png",
                f"build/usr/share/icons/hicolor/{res}x{res}/apps/fig.png",
            )

    info("Copying bundle files")
    bin_path = pathlib.Path("build/usr/bin")
    bin_path.mkdir(parents=True, exist_ok=True)
    shutil.copy2(cw_cli_path, bin_path)
    shutil.copy2(cwterm_path, bin_path)
    shutil.copytree("bundle/linux/minimal", BUILD_DIR, dirs_exist_ok=True)
    if not is_minimal:
        shutil.copytree("bundle/linux/desktop", BUILD_DIR, dirs_exist_ok=True)
        shutil.copy2(codewhisperer_desktop_path, bin_path)


def generate_sha(path: pathlib.Path) -> pathlib.Path:
    if isDarwin():
        shasum_output = run_cmd_output(["shasum", "-a", "256", path])
    elif isLinux():
        shasum_output = run_cmd_output(["sha256sum", path])
    else:
        raise Exception("Unsupported platform")

    sha = shasum_output.split(" ")[0]
    path = path.with_name(f"{path.name}.sha256")
    path.write_text(sha)
    info(f"Wrote sha256sum to {path}:", sha)
    return path


# parse argv[1] for json and build into dict, then grab each known value
build_args = {
    k: v
    for (k, v) in json.loads(sys.argv[1] if len(sys.argv) > 1 else "{}").items()
    if isinstance(k, str) and isinstance(v, str)
}

info(f"Build Args: {build_args}")

output_bucket = build_args.get("output_bucket")
signing_bucket = build_args.get("signing_bucket")
aws_account_id = build_args.get("aws_account_id")
apple_id_secret = build_args.get("apple_id_secret")
signing_queue = build_args.get("signing_queue")
signing_role_name = build_args.get("signing_role_name")
stage_name = build_args.get("stage_name")
run_lints = build_args.get("run_lints")
variant = get_variant()

if signing_bucket and aws_account_id and apple_id_secret and signing_queue and signing_role_name:
    signing_data = EcSigningData(
        bucket_name=signing_bucket,
        aws_account_id=aws_account_id,
        notarizing_secret_id=apple_id_secret,
        signing_request_queue_name=signing_queue,
        signing_role_name=signing_role_name,
    )
else:
    signing_data = None

cargo_features: Mapping[str, Sequence[str]] = {}

if stage_name == "prod" or stage_name is None:
    info("Building for prod")
elif stage_name == "gamma":
    info("Building for gamma")
    cargo_features = {"cw_cli": ["cw_cli/gamma"], "fig_desktop": ["fig_desktop/gamma"]}
else:
    raise ValueError(f"Unknown stage name: {stage_name}")

info(f"Cargo features: {cargo_features}")
info(f"Signing app: {signing_data is not None}")
info(f"Variant: {variant.name}")

BUILD_DIR.mkdir(parents=True, exist_ok=True)

if variant == Variant.FULL:
    info("Building npm packages")
    npm_packages = build_npm_packages()

info("Running cargo tests")
run_cargo_tests(features=cargo_features)

if run_lints:
    run_clippy(features=cargo_features)

info("Building cw_cli")
cw_cli_path = build_cargo_bin("cw_cli", output_name="cw", features=cargo_features)

info("Building figterm")
cwterm_path = build_cargo_bin("figterm", output_name="cwterm", features=cargo_features)

if isDarwin():
    info("Building CodeWhisperer.dmg")
    dmg_path = build_desktop_app(
        cw_cli_path=cw_cli_path,
        cwterm_path=cwterm_path,
        npm_packages=npm_packages,
        signing_data=signing_data,
        features=cargo_features,
    )

    sha_path = generate_sha(dmg_path)

    if output_bucket:
        staging_location = f"s3://{build_args['output_bucket']}/staging/"
        info(f"Build complete, sending to {staging_location}")

        run_cmd(["aws", "s3", "cp", dmg_path, staging_location])
        run_cmd(["aws", "s3", "cp", sha_path, staging_location])
elif isLinux():
    # create the archive structure:
    #   amazon-q/bin/cw
    #   amazon-q/bin/cwterm
    #   amazon-q/install.sh

    archive_name = "amazon-q"

    archive_path = pathlib.Path(archive_name)
    archive_path.mkdir(parents=True, exist_ok=True)

    shutil.copy2("bundle/linux/install.sh", archive_path)

    archive_bin_path = archive_path / "bin"
    archive_bin_path.mkdir(parents=True, exist_ok=True)

    shutil.copy2(cw_cli_path, archive_bin_path)
    shutil.copy2(cwterm_path, archive_bin_path)

    signer = load_gpg_signer()

    info(f"Building {archive_name}.tar.gz")
    tar_gz_path = BUILD_DIR / f"{archive_name}.tar.gz"
    run_cmd(["tar", "-czf", tar_gz_path, archive_path])
    generate_sha(tar_gz_path)
    if signer:
        signer.sign_file(tar_gz_path)

    info(f"Building {archive_name}.tar.xz")
    tar_xz_path = BUILD_DIR / f"{archive_name}.tar.xz"
    run_cmd(["tar", "-cJf", tar_xz_path, archive_path])
    generate_sha(tar_xz_path)
    if signer:
        signer.sign_file(tar_xz_path)

    info(f"Building {archive_name}.tar.zst")
    tar_zst_path = BUILD_DIR / f"{archive_name}.tar.zst"
    run_cmd(["tar", "-I", "zstd", "-cf", tar_zst_path, archive_path], {"ZSTD_CLEVEL": "19"})
    generate_sha(tar_zst_path)
    if signer:
        signer.sign_file(tar_zst_path)

    info(f"Building {archive_name}.zip")
    zip_path = BUILD_DIR / f"{archive_name}.zip"
    run_cmd(["zip", "-r", zip_path, archive_path])
    generate_sha(zip_path)
    if signer:
        signer.sign_file(zip_path)

    # clean up
    shutil.rmtree(archive_path)
    if signer:
        signer.clean()
