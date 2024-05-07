import argparse
import os
import subprocess
from build import build
from const import CLI_PACKAGE_NAME
from doc import run_doc
from rust import rust_env
from test import all_tests


class StoreIfNotEmptyAction(argparse.Action):
    def __call__(self, parser, namespace, values, option_string=None):
        if values and len(values) > 0:
            setattr(namespace, self.dest, values)


parser = argparse.ArgumentParser(
    prog="build",
    description="Builds the FigIoDesktop application",
)
subparsers = parser.add_subparsers(help="sub-command help", dest="subparser", required=True)

build_subparser = subparsers.add_parser(name="build")
build_subparser.add_argument(
    "--output-bucket",
    action=StoreIfNotEmptyAction,
    help="The name of bucket to store the build artifacts",
)
build_subparser.add_argument(
    "--signing-bucket",
    action=StoreIfNotEmptyAction,
    help="The name of bucket to store the build artifacts",
)
build_subparser.add_argument(
    "--aws-account-id",
    action=StoreIfNotEmptyAction,
    help="The AWS account ID",
)
build_subparser.add_argument(
    "--apple-id-secret",
    action=StoreIfNotEmptyAction,
    help="The Apple ID secret",
)
build_subparser.add_argument(
    "--signing-queue",
    action=StoreIfNotEmptyAction,
    help="The name of the signing queue",
)
build_subparser.add_argument(
    "--signing-role-name",
    action=StoreIfNotEmptyAction,
    help="The name of the signing role",
)
build_subparser.add_argument(
    "--stage-name",
    action=StoreIfNotEmptyAction,
    help="The name of the stage",
)
build_subparser.add_argument(
    "--run-lints",
    action=StoreIfNotEmptyAction,
    help="Run lints",
)
build_subparser.add_argument(
    "--not-release",
    action="store_true",
    help="Build a non-release version",
)

test_subparser = subparsers.add_parser(name="test")
test_subparser.add_argument(
    "--clippy-fail-on-warn",
    action="store_true",
    help="Fail on clippy warnings",
)

# runs CLI with the given arguments
cli_subparser = subparsers.add_parser(name="cli")
cli_subparser.add_argument(
    "args",
    nargs=argparse.REMAINDER,
    help="Arguments to pass to the CLI",
)

# run the docs command
subparsers.add_parser(name="doc")

args = parser.parse_args()

match args.subparser:
    case "build":
        build(
            release=not args.not_release,
            output_bucket=args.output_bucket,
            signing_bucket=args.signing_bucket,
            aws_account_id=args.aws_account_id,
            apple_id_secret=args.apple_id_secret,
            signing_queue=args.signing_queue,
            signing_role_name=args.signing_role_name,
            stage_name=args.stage_name,
            run_lints=args.run_lints,
        )
    case "test":
        all_tests(
            clippy_fail_on_warn=args.clippy_fail_on_warn,
        )
    case "doc":
        run_doc()
    case "cli":
        subprocess.run(
            [
                "cargo",
                "run",
                f"--bin={CLI_PACKAGE_NAME}",
                *args.args,
            ],
            env={
                **os.environ,
                **rust_env(release=False),
            },
        )
    case _:
        raise ValueError(f"Unsupported subparser {args.subparser}")
