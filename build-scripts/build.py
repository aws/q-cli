from dataclasses import dataclass
from functools import cache
import os
import json
import datetime
import pathlib
import shutil
from typing import Mapping, Sequence
from util import Variant, get_variant, isDarwin, isLinux, run_cmd, run_cmd_output, info, version
from rust import build_hash, build_datetime, cargo_cmd_name, rust_targets, rust_env, get_target_triple
from test import run_cargo_tests, run_clippy
from signing import (
    CdSigningData,
    CdSigningType,
    load_gpg_signer,
    rebundle_dmg,
    cd_sign_file,
    apple_notarize_file,
)
from importlib import import_module
from const import (
    APP_NAME,
    CLI_BINARY_NAME,
    CLI_PACKAGE_NAME,
    DESKTOP_PACKAGE_NAME,
    DMG_NAME,
    LINUX_ARCHIVE_NAME,
    MACOS_BUNDLE_ID,
    PTY_BINARY_NAME,
    PTY_PACKAGE_NAME,
    URL_SCHEMA,
)

BUILD_DIR_RELATIVE = pathlib.Path(os.environ.get("BUILD_DIR") or "build")
BUILD_DIR = BUILD_DIR_RELATIVE.absolute()


@dataclass
class NpmBuildOutput:
    dashboard_path: pathlib.Path
    autocomplete_path: pathlib.Path
    vscode_path: pathlib.Path


def build_npm_packages() -> NpmBuildOutput:
    run_cmd(["pnpm", "install", "--frozen-lockfile"])

    # set the version of extensions/vscode
    package_json_path = pathlib.Path("extensions/vscode/package.json")
    packge_json_text = package_json_path.read_text()
    packge_json = json.loads(packge_json_text)
    packge_json["version"] = version()
    package_json_path.write_text(json.dumps(packge_json, indent=2))

    run_cmd(["pnpm", "build"])
    run_cmd(["pnpm", "test", "--", "--run"])

    # revert the package.json
    package_json_path.write_text(packge_json_text)

    # copy to output
    dashboard_path = BUILD_DIR / "dashboard"
    shutil.rmtree(dashboard_path, ignore_errors=True)
    shutil.copytree("apps/dashboard/dist", dashboard_path)

    autocomplete_path = BUILD_DIR / "autocomplete"
    shutil.rmtree(autocomplete_path, ignore_errors=True)
    shutil.copytree("apps/autocomplete/dist", autocomplete_path)

    vscode_path = BUILD_DIR / "vscode-plugin.vsix"
    shutil.rmtree(vscode_path, ignore_errors=True)
    shutil.copy2(f"extensions/vscode/codewhisperer-for-command-line-companion-{version()}.vsix", vscode_path)
    shutil.copy2(
        f"extensions/vscode/codewhisperer-for-command-line-companion-{version()}.vsix",
        "lib/fig_integrations/src/vscode/vscode-plugin.vsix",
    )

    return NpmBuildOutput(dashboard_path=dashboard_path, autocomplete_path=autocomplete_path, vscode_path=vscode_path)


def build_cargo_bin(
    release: bool,
    package: str,
    output_name: str | None = None,
    features: Mapping[str, Sequence[str]] | None = None,
    targets: Sequence[str] = [],
) -> pathlib.Path:
    args = [cargo_cmd_name(), "build", "--locked", "--package", package]

    if release:
        args.append("--release")

    for target in targets:
        args.extend(["--target", target])

    if features and features.get(package):
        args.extend(["--features", ",".join(features[package])])

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(release=release),
        },
    )

    if release:
        target_subdir = "release"
    else:
        target_subdir = "debug"

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
            args.append(pathlib.Path("target") / target / target_subdir / package)
        run_cmd(args)
        return out_path
    else:
        # linux does not cross compile arch
        target = targets[0]
        target_path = pathlib.Path("target") / target / target_subdir / package
        out_path = BUILD_DIR / "bin" / (output_name or package)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(target_path, out_path)
        return out_path


@cache
def gen_manifest() -> str:
    return json.dumps(
        {
            "managed_by": "dmg",
            "packaged_at": datetime.datetime.now().isoformat(),
            "packaged_by": "amazon",
            "variant": "full",
            "version": version(),
            "kind": "dmg",
            "default_channel": "stable",
        }
    )


def build_macos_ime(
    release: bool,
    is_prod: bool,
    signing_data: CdSigningData | None,
    targets: Sequence[str] = [],
) -> pathlib.Path:
    fig_input_method_bin = build_cargo_bin(release=release, package="fig_input_method", targets=targets)
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
        cd_sign_file(input_method_app, CdSigningType.IME, signing_data, is_prod=is_prod)
        apple_notarize_file(input_method_app, signing_data)

    return input_method_app


def tauri_config(cli_path: pathlib.Path, pty_path: pathlib.Path, target: str) -> str:
    config = {
        "tauri": {
            "bundle": {
                "externalBin": [
                    str(cli_path).removesuffix(f"-{target}"),
                    str(pty_path).removesuffix(f"-{target}"),
                ],
                "resources": ["manifest.json"],
            }
        }
    }
    return json.dumps(config)


def build_desktop_app(
    release: bool,
    pty_path: pathlib.Path,
    cli_path: pathlib.Path,
    npm_packages: NpmBuildOutput,
    is_prod: bool,
    signing_data: CdSigningData | None,
    features: Mapping[str, Sequence[str]] | None = None,
    targets: Sequence[str] = [],
) -> pathlib.Path:
    target = get_target_triple()

    info("Building macos ime")
    ime_app = build_macos_ime(release=release, signing_data=signing_data, targets=targets, is_prod=is_prod)

    info("Writing manifest")
    manifest_path = pathlib.Path(DESKTOP_PACKAGE_NAME) / "manifest.json"
    manifest_path.write_text(gen_manifest())

    info("Building tauri config")
    tauri_config_path = pathlib.Path(DESKTOP_PACKAGE_NAME) / "build-config.json"
    tauri_config_path.write_text(tauri_config(cli_path=cli_path, pty_path=pty_path, target=target))

    info("Building", DESKTOP_PACKAGE_NAME)

    cargo_tauri_args = [
        "cargo-tauri",
        "build",
        "--config",
        "build-config.json",
        "--target",
        target,
    ]

    if features and features.get(DESKTOP_PACKAGE_NAME):
        cargo_tauri_args.extend(["--features", ",".join(features[DESKTOP_PACKAGE_NAME])])

    run_cmd(
        cargo_tauri_args,
        cwd=DESKTOP_PACKAGE_NAME,
        env={**os.environ, **rust_env(release=release), "BUILD_DIR": BUILD_DIR},
    )

    # clean up
    manifest_path.unlink(missing_ok=True)
    tauri_config_path.unlink(missing_ok=True)

    target_bundle = pathlib.Path(f"target/{target}/release/bundle/macos/q_desktop.app")
    app_path = BUILD_DIR / f"{APP_NAME}.app"
    shutil.rmtree(app_path, ignore_errors=True)
    shutil.copytree(target_bundle, app_path)

    info_plist_path = app_path / "Contents/Info.plist"

    # Change the display name of the app
    run_cmd(["defaults", "write", info_plist_path, "CFBundleDisplayName", APP_NAME])
    run_cmd(["defaults", "write", info_plist_path, "CFBundleName", APP_NAME])

    # Specifies the app is an "agent app"
    run_cmd(["defaults", "write", info_plist_path, "LSUIElement", "-bool", "TRUE"])

    # Add q:// association to bundle
    run_cmd(
        [
            "plutil",
            "-insert",
            "CFBundleURLTypes",
            "-xml",
            f"""<array>
    <dict>
        <key>CFBundleURLName</key>
        <string>{MACOS_BUNDLE_ID}</string>
        <key>CFBundleURLSchemes</key>
        <array>
        <string>{URL_SCHEMA}</string>
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

    info("Copying dashboard into bundle")
    shutil.copytree(npm_packages.dashboard_path, app_path / "Contents/Resources/dashboard")

    info("Copying autocomplete into bundle")
    shutil.copytree(npm_packages.autocomplete_path, app_path / "Contents/Resources/autocomplete")

    # Add symlinks
    # os.symlink(f"./{CLI_BINARY_NAME}", app_path / "Contents/MacOS/cli")
    # os.symlink(f"./{PTY_BINARY_NAME}", app_path / "Contents/MacOS/pty")

    dmg_path = BUILD_DIR / f"{DMG_NAME}.dmg"
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
        sign_and_rebundle_macos(app_path=app_path, dmg_path=dmg_path, signing_data=signing_data, is_prod=is_prod)

    return dmg_path


def sign_and_rebundle_macos(app_path: pathlib.Path, dmg_path: pathlib.Path, signing_data: CdSigningData, is_prod: bool):
    info("Signing app and dmg")

    # Sign the application
    cd_sign_file(app_path, CdSigningType.APP, signing_data, is_prod=is_prod)

    # Notarize the application

    apple_notarize_file(app_path, signing_data)

    # Rebundle the dmg file with the signed and notarized application
    rebundle_dmg(app_path=app_path, dmg_path=dmg_path)

    # Sign the dmg
    cd_sign_file(dmg_path, CdSigningType.DMG, signing_data, is_prod=is_prod)

    # Notarize the dmg
    apple_notarize_file(dmg_path, signing_data)

    info("Done signing!!")


def linux_bundle(
    pty_path: pathlib.Path,
    cli_path: pathlib.Path,
    desktop_app_path: pathlib.Path,
    is_minimal: bool,
):
    if not is_minimal:
        for res in [16, 22, 24, 32, 48, 64, 128, 256, 512]:
            shutil.copy2(
                f"{DESKTOP_PACKAGE_NAME}/icons/{res}x{res}.png",
                f"build/usr/share/icons/hicolor/{res}x{res}/apps/fig.png",
            )

    info("Copying bundle files")
    bin_path = pathlib.Path("build/usr/bin")
    bin_path.mkdir(parents=True, exist_ok=True)
    shutil.copy2(cli_path, bin_path)
    shutil.copy2(pty_path, bin_path)
    shutil.copytree("bundle/linux/minimal", BUILD_DIR, dirs_exist_ok=True)
    if not is_minimal:
        shutil.copytree("bundle/linux/desktop", BUILD_DIR, dirs_exist_ok=True)
        shutil.copy2(desktop_app_path, bin_path)


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


@dataclass
class BuildOutput:
    cli_path: pathlib.Path
    pty_path: pathlib.Path


def build(
    release: bool,
    output_bucket: str | None = None,
    signing_bucket: str | None = None,
    aws_account_id: str | None = None,
    apple_id_secret: str | None = None,
    signing_role_name: str | None = None,
    stage_name: str | None = None,
    run_lints: bool = True,
    run_test: bool = True,
):
    variant = get_variant()

    if signing_bucket and aws_account_id and apple_id_secret and signing_role_name:
        signing_data = CdSigningData(
            bucket_name=signing_bucket,
            aws_account_id=aws_account_id,
            notarizing_secret_id=apple_id_secret,
            signing_role_name=signing_role_name,
        )
    else:
        signing_data = None

    cargo_features: Mapping[str, Sequence[str]] = {}
    match stage_name:
        case "prod" | None:
            info("Building for prod")
        case "gamma":
            info("Building for gamma")
            cargo_features = {
                CLI_PACKAGE_NAME: [f"{CLI_PACKAGE_NAME}/gamma"],
                DESKTOP_PACKAGE_NAME: [f"{DESKTOP_PACKAGE_NAME}/gamma"],
            }
        case _:
            raise ValueError(f"Unknown stage name: {stage_name}")

    info(f"Release: {release}")
    info(f"Cargo features: {cargo_features}")
    info(f"Signing app: {signing_data is not None}")
    info(f"Variant: {variant.name}")

    BUILD_DIR.mkdir(parents=True, exist_ok=True)

    if variant == Variant.FULL:
        info("Building npm packages")
        npm_packages = build_npm_packages()

    targets = rust_targets()

    # Mac has multiple targets, so just use the default for the platform
    # for testing and linting.
    cargo_test_target = None if isDarwin() else targets[0]

    if run_test:
        info("Running cargo tests")
        run_cargo_tests(features=cargo_features, target=cargo_test_target)

    if run_lints:
        run_clippy(features=cargo_features, target=cargo_test_target)

    info("Building", CLI_PACKAGE_NAME)
    cli_path = build_cargo_bin(
        release=release, package=CLI_PACKAGE_NAME, output_name=CLI_BINARY_NAME, features=cargo_features, targets=targets
    )

    info("Building", PTY_PACKAGE_NAME)
    pty_path = build_cargo_bin(
        release=release, package=PTY_PACKAGE_NAME, output_name=PTY_BINARY_NAME, features=cargo_features, targets=targets
    )

    if isDarwin():
        info(f"Building {DMG_NAME}.dmg")
        dmg_path = build_desktop_app(
            release=release,
            cli_path=cli_path,
            pty_path=pty_path,
            npm_packages=npm_packages,
            signing_data=signing_data,
            features=cargo_features,
            targets=targets,
            is_prod=stage_name == "prod" or stage_name is None,
        )

        sha_path = generate_sha(dmg_path)

        if output_bucket:
            staging_location = f"s3://{output_bucket}/staging/"
            info(f"Build complete, sending to {staging_location}")

            run_cmd(["aws", "s3", "cp", dmg_path, staging_location])
            run_cmd(["aws", "s3", "cp", sha_path, staging_location])
    elif isLinux():
        # create the archive structure:
        #   archive/bin/q
        #   archive/bin/qterm
        #   archive/install.sh
        #   archive/README
        #   archive/BUILD-INFO

        archive_name = LINUX_ARCHIVE_NAME

        archive_path = pathlib.Path(archive_name)
        archive_path.mkdir(parents=True, exist_ok=True)

        shutil.copy2("bundle/linux/install.sh", archive_path)
        shutil.copy2("bundle/linux/README", archive_path)

        # write the BUILD-INFO
        build_info_path = archive_path / "BUILD-INFO"
        build_info_path.write_text(
            "\n".join(
                [
                    f"BUILD_DATE={build_datetime()}",
                    f"BUILD_HASH={build_hash()}",
                    f"BUILD_TARGET_TRIPLE={get_target_triple()}",
                    f"BUILD_VERSION={version}",
                ]
            )
        )

        archive_bin_path = archive_path / "bin"
        archive_bin_path.mkdir(parents=True, exist_ok=True)

        shutil.copy2(cli_path, archive_bin_path)
        shutil.copy2(pty_path, archive_bin_path)

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

    return BuildOutput(
        cli_path=cli_path,
        pty_path=pty_path,
    )
