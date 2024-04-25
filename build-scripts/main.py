import argparse
from build import build
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

test_subparser = subparsers.add_parser(name="test")
test_subparser.add_argument(
    "--clippy-fail-on-warn",
    action="store_true",
    help="Fail on clippy warnings",
)

args = parser.parse_args()

match args.subparser:
    case "build":
        build(
            release=True,
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
    case _:
        raise ValueError(f"Unsupported subparser {args.subparser}")
