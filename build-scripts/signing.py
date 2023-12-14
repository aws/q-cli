import pathlib
from util import info, run_cmd, run_cmd_output, run_cmd_status
from enum import Enum
import json
import shutil
import time

TEAM_ID = "94KV3E626L"


class SigningData:
    bucket_name: str
    signing_request_queue_name: str
    notarizing_secret_id: str
    aws_account_id: str
    signing_role_name: str

    def __init__(
        self,
        bucket_name: str,
        signing_request_queue_name: str,
        notarizing_secret_id: str,
        aws_account_id: str,
        signing_role_name: str,
    ):
        self.bucket_name = bucket_name
        self.signing_request_queue_name = signing_request_queue_name
        self.notarizing_secret_id = notarizing_secret_id
        self.aws_account_id = aws_account_id
        self.signing_role_name = signing_role_name


def signed_package_exists(name: str, signing_data: SigningData) -> bool:
    s3_path = f"s3://{signing_data.bucket_name}/{name}"
    info(f"Checking if {s3_path} exists")
    return run_cmd_status(["aws", "s3", "ls", s3_path]) == 0


def post_request(source: str, destination: str, signing_data: SigningData):
    message = {
        "data": {
            "source": {"arn": f"arn:aws:s3:::{source}"},
            "destination": {"arn": f"arn:aws:s3:::{destination}"},
            "iam-role": {
                "arn": f"arn:aws:iam::{signing_data.aws_account_id}:role/{signing_data.signing_role_name}"
            },
        }
    }
    message_json = json.dumps(message)

    queue_url_output = run_cmd_output(
        [
            "aws",
            "sqs",
            "get-queue-url",
            "--queue-name",
            signing_data.signing_request_queue_name,
        ]
    )
    queue_url = json.loads(queue_url_output)["QueueUrl"]

    run_cmd(
        [
            "aws",
            "sqs",
            "send-message",
            "--queue-url",
            queue_url,
            "--message-body",
            message_json,
        ]
    )


class SigningType(Enum):
    DMG = "dmg"
    APP = "app"
    IME = "ime"


def build_signed_package(type: SigningType, file_path: pathlib.Path, name: str):
    working_dir = pathlib.Path(f"build-config/signing/{type.value}")
    starting_dir = pathlib.Path.cwd()

    if type == SigningType.DMG:
        # Our dmg file names vary by platform, so this is templated in the manifest
        manifest_template_path = working_dir / "manifest.yaml.template"
        manifest_path = working_dir / "manifest.yaml"
        manifest_path.write_text(
            manifest_template_path.read_text().replace("__NAME__", name)
        )

    if file_path.is_dir():
        shutil.copytree(file_path, working_dir / "artifact" / file_path.name)
        shutil.rmtree(file_path)
    elif file_path.is_file():
        shutil.copy2(file_path, working_dir / "artifact" / file_path.name)
        file_path.unlink()
    else:
        raise Exception(f"Unknown file type: {file_path}")

    run_cmd(
        [
            "gtar",
            "-czf",
            working_dir / "artifact.gz",
            "-C",
            working_dir / "artifact",
            ".",
        ]
    )
    run_cmd(
        [
            "gtar",
            "-czf",
            starting_dir / "package.tar.gz",
            "manifest.yaml",
            "artifact.gz",
        ],
        cwd=working_dir,
    )
    (working_dir / "artifact.gz").unlink()
    shutil.rmtree(working_dir / "artifact")


def sign_file(file: pathlib.Path, type: SigningType, signing_data: SigningData):
    name = file.name

    info(f"Signing {name}")

    # Electric Company requires us to build up a tar file in an extremely specific format
    info("Packaging...")
    build_signed_package(type, file, name)

    # Upload package for signing to S3
    info("Uploading...")
    run_cmd(
        [
            "aws",
            "s3",
            "rm",
            "--recursive",
            f"s3://{signing_data.bucket_name}/signed",
        ]
    )
    run_cmd(
        [
            "aws",
            "s3",
            "rm",
            "--recursive",
            f"s3://{signing_data.bucket_name}/pre-signed",
        ]
    )
    run_cmd(
        [
            "aws",
            "s3",
            "cp",
            "package.tar.gz",
            f"s3://{signing_data.bucket_name}/pre-signed/package.tar.gz",
        ]
    )
    pathlib.Path("package.tar.gz").unlink()

    info("Sending request...")
    post_request(
        f"{signing_data.bucket_name}/pre-signed/package.tar.gz",
        f"{signing_data.bucket_name}/signed/signed.zip",
        signing_data,
    )

    # Loop until the signed package appears in the S3 bucket, for a maximum of 3 minutes
    max_duration = 180
    end_time = time.time() + max_duration
    i = 1
    while True:
        info(f"Checking for signed package {i}")
        if signed_package_exists("signed/signed.zip", signing_data):
            break
        if time.time() >= end_time:
            raise RuntimeError("Signed package did not appear, check signer logs")
        time.sleep(10)
        i += 1

    info("Signed!")

    info("Downloading...")
    run_cmd(
        [
            "aws",
            "s3",
            "cp",
            f"s3://{signing_data.bucket_name}/signed/signed.zip",
            "signed.zip",
        ]
    )
    run_cmd(["unzip", "signed.zip"])

    # find child of Payload
    children = list(pathlib.Path("Payload").iterdir())
    if len(children) != 1:
        raise RuntimeError("Payload directory should have exactly one child")

    child_path = children[0]

    # copy child to the original file location
    if child_path.is_dir():
        shutil.copytree(child_path, file)
    elif child_path.is_file():
        shutil.copy2(child_path, file)
    else:
        raise Exception(f"Unknown file type: {child_path}")

    # clean up
    pathlib.Path("signed.zip").unlink()
    shutil.rmtree("Payload")

    info(f"Signing status of {file}")
    run_cmd(["codesign", "-dv", "--deep", "--strict", file])


def rebundle_dmg(dmg_path: pathlib.Path, app_path: pathlib.Path):
    mounting_path = pathlib.Path("/Volumes/CodeWhisperer")

    info(f"Rebunding {dmg_path}")

    # Try to unmount a dmg if it is already there
    if mounting_path.is_dir():
        run_cmd(["hdiutil", "detach", mounting_path])

    tempdmg_path = pathlib.Path.home() / "temp.dmg"
    tempdmg_path.unlink(missing_ok=True)

    # Convert the dmg to writable
    run_cmd(["hdiutil", "convert", dmg_path, "-format", "UDRW", "-o", tempdmg_path])

    # Mount the dmg
    run_cmd(["hdiutil", "attach", tempdmg_path])

    # Copy in the new app
    # shutil.copytree(app_path, mounting_path, dirs_exist_ok=True)
    run_cmd(["cp", "-R", app_path, mounting_path])

    # Unmount the dmg
    run_cmd(["hdiutil", "detach", mounting_path])

    # Convert the dmg to zipped, read only - this is the only type that EC will accept!!
    dmg_path.unlink()
    run_cmd(
        [
            "hdiutil",
            "convert",
            tempdmg_path,
            "-format",
            "UDZO",
            "-o",
            dmg_path,
        ]
    )


def notarize_file(file: pathlib.Path, signing_data: SigningData):
    name = file.name
    file_type = file.suffix[1:]

    file_to_notarize = file

    if file_type == "app":
        # We can submit dmg files as is, but we have to zip up app files in a specific way
        file_to_notarize = pathlib.Path(f"{name}.zip")
        run_cmd(
            [
                "ditto",
                "-c",
                "-k",
                "--sequesterRsrc",
                "--keepParent",
                file,
                file_to_notarize,
            ]
        )

    secrets = get_signing_secrets(signing_data.notarizing_secret_id)

    run_cmd(
        [
            "xcrun",
            "notarytool",
            "submit",
            file_to_notarize,
            "--team-id",
            TEAM_ID,
            "--apple-id",
            secrets["appleId"],
            "--password",
            secrets["appleIdPassword"],
            "--wait",
        ]
    )

    run_cmd(
        [
            "xcrun",
            "stapler",
            "staple",
            file,
        ]
    )

    if file_type == "app":
        # Verify notarization for .app
        run_cmd(
            [
                "spctl",
                "-a",
                "-v",
                file,
            ]
        )

        pathlib.Path(file_to_notarize).unlink()
    else:
        # Verify notarization for .dmg
        run_cmd(
            [
                "spctl",
                "-a",
                "-t",
                "open",
                "--context",
                "context:primary-signature",
                "-v",
                file,
            ]
        )


def get_signing_secrets(notarizing_secret_id: str):
    secret_string = run_cmd_output(
        [
            "aws",
            "secretsmanager",
            "get-secret-value",
            "--secret-id",
            notarizing_secret_id,
        ]
    )
    secret_string = json.loads(secret_string)["SecretString"]
    return json.loads(secret_string)
