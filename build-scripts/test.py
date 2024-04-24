import itertools
import os
from typing import List, Mapping, Sequence
from rust import rust_env
from util import isLinux, run_cmd
from const import DESKTOP_PACKAGE_NAME


def run_clippy(
    features: Mapping[str, Sequence[str]] | None = None, targets: List[str] = [], fail_on_warn: bool = False
):
    # run clippy using the same config as normal builds
    args = ["cargo", "clippy", "--locked"]

    for target in targets:
        args.extend(["--target", target])

    if isLinux():
        args.extend(["--workspace", "--exclude", DESKTOP_PACKAGE_NAME])

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
    # build the tests using the same config as normal builds
    args = ["cargo", "build", "--tests", "--locked"]

    for target in targets:
        args.extend(["--target", target])

    if isLinux():
        args.extend(["--workspace", "--exclude", DESKTOP_PACKAGE_NAME])

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

    args = ["cargo", "test", "--locked"]

    # disable desktop tests for now
    if isLinux():
        args.extend(["--workspace", "--exclude", DESKTOP_PACKAGE_NAME])

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


def all_tests(clippy_fail_on_warn: bool):
    run_cargo_tests()
    run_clippy(fail_on_warn=clippy_fail_on_warn)
