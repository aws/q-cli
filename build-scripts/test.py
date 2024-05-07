import itertools
import os
from typing import List, Mapping, Sequence
from rust import rust_env
from util import isBrazil, isLinux, run_cmd
from const import CLI_PACKAGE_NAME, DESKTOP_PACKAGE_NAME, PTY_PACKAGE_NAME


def run_clippy(
    features: Mapping[str, Sequence[str]] | None = None, targets: List[str] = [], fail_on_warn: bool = False
):
    args = ["cargo", "clippy", "--locked", "--workspace"]

    for target in targets:
        args.extend(["--target", target])

    if isLinux() or isBrazil():
        args.extend(["--exclude", DESKTOP_PACKAGE_NAME])

    if features:
        args.extend(
            [
                "--features",
                ",".join(set(itertools.chain.from_iterable(features.values()))),
            ]
        )

    if fail_on_warn:
        args.extend(["--", "-D", "warnings"])

    run_cmd(
        args,
        env={
            **os.environ,
            **rust_env(release=False),
        },
    )


def run_cargo_tests(features: Mapping[str, Sequence[str]] | None = None, targets: List[str] = []):
    args = [
        "cargo",
    ]

    if isBrazil():
        args.extend(["brazil", "with-coverage"])

    args.extend(["build", "--tests", "--locked", "--workspace"])

    for target in targets:
        args.extend(["--target", target])

    if isLinux() or isBrazil():
        args.extend(["--exclude", DESKTOP_PACKAGE_NAME])

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
            **rust_env(release=False),
        },
    )

    args = ["cargo"]

    if isBrazil():
        args.extend(["brazil", "with-coverage"])

    args.extend(["test", "--locked", "--workspace"])

    # disable desktop tests for now
    if isLinux() or isBrazil():
        args.extend(["--exclude", DESKTOP_PACKAGE_NAME])

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
            **rust_env(release=False),
        },
    )

    if isBrazil():
        run_cmd(
            [
                "cargo",
                "brazil",
                "with-coverage",
                "report",
                "--",
                "--branch",
                "--ignore-not-existing",
                "--excl-start",
                r"// GRCOV_STOP_COVERAGE",
                "--excl-stop",
                r"// GRCOV_BEGIN_COVERAGE",
                "--excl-line",
                r"// GRCOV_IGNORE_LINE",
                "--keep-only",
                f"{CLI_PACKAGE_NAME}/**/*.rs",
                "--keep-only",
                f"{PTY_PACKAGE_NAME}/**/*.rs",
                "--keep-only",
                f"{DESKTOP_PACKAGE_NAME}/**/*.rs",
                "--keep-only",
                "lib/**/*.rs",
                "--ignore",
                "lib/amzn-*/**/*.rs",
            ]
        )


def all_tests(clippy_fail_on_warn: bool):
    run_cargo_tests()
    run_clippy(fail_on_warn=clippy_fail_on_warn)
