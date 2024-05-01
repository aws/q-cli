import base64
import os
import pathlib
from typing import List, Optional
from util import Args, Env, info, run_cmd, run_cmd_output, run_cmd_status
from enum import Enum
import json
import shutil
import time

APPLE_TEAM_ID = "94KV3E626L"


class EcSigningData:
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


def ec_signed_package_exists(name: str, signing_data: EcSigningData) -> bool:
    s3_path = f"s3://{signing_data.bucket_name}/{name}"
    info(f"Checking if {s3_path} exists")
    return run_cmd_status(["aws", "s3", "ls", s3_path]) == 0


def ec_post_request(source: str, destination: str, signing_data: EcSigningData):
    message = {
        "data": {
            "source": {"arn": f"arn:aws:s3:::{source}"},
            "destination": {"arn": f"arn:aws:s3:::{destination}"},
            "iam-role": {"arn": f"arn:aws:iam::{signing_data.aws_account_id}:role/{signing_data.signing_role_name}"},
        }
    }
    message_json = json.dumps(message)

    queue_url_output = run_cmd_output(
        ["aws", "sqs", "get-queue-url", "--queue-name", signing_data.signing_request_queue_name]
    )
    queue_url = json.loads(queue_url_output)["QueueUrl"]

    run_cmd(["aws", "sqs", "send-message", "--queue-url", queue_url, "--message-body", message_json])


class EcSigningType(Enum):
    DMG = "dmg"
    APP = "app"
    IME = "ime"


def ec_build_signed_package(type: EcSigningType, file_path: pathlib.Path, name: str):
    working_dir = pathlib.Path(f"build-config/signing/{type.value}")
    starting_dir = pathlib.Path.cwd()

    if type == EcSigningType.DMG:
        # Our dmg file names vary by platform, so this is templated in the manifest
        manifest_template_path = working_dir / "manifest.yaml.template"
        manifest_path = working_dir / "manifest.yaml"
        manifest_path.write_text(manifest_template_path.read_text().replace("__NAME__", name))

    if file_path.is_dir():
        shutil.copytree(file_path, working_dir / "artifact" / file_path.name)
        shutil.rmtree(file_path)
    elif file_path.is_file():
        shutil.copy2(file_path, working_dir / "artifact" / file_path.name)
        file_path.unlink()
    else:
        raise Exception(f"Unknown file type: {file_path}")

    run_cmd(["gtar", "-czf", working_dir / "artifact.gz", "-C", working_dir / "artifact", "."])
    run_cmd(
        ["gtar", "-czf", starting_dir / "package.tar.gz", "manifest.yaml", "artifact.gz"],
        cwd=working_dir,
    )
    (working_dir / "artifact.gz").unlink()
    shutil.rmtree(working_dir / "artifact")


# Sign a file with electric company
def ec_sign_file(file: pathlib.Path, type: EcSigningType, signing_data: EcSigningData):
    name = file.name

    info(f"Signing {name}")

    # Electric Company requires us to build up a tar file in an extremely specific format
    info("Packaging...")
    ec_build_signed_package(type, file, name)

    # Upload package for signing to S3
    info("Uploading...")
    run_cmd(["aws", "s3", "rm", "--recursive", f"s3://{signing_data.bucket_name}/signed"])
    run_cmd(["aws", "s3", "rm", "--recursive", f"s3://{signing_data.bucket_name}/pre-signed"])
    run_cmd(["aws", "s3", "cp", "package.tar.gz", f"s3://{signing_data.bucket_name}/pre-signed/package.tar.gz"])
    pathlib.Path("package.tar.gz").unlink()

    info("Sending request...")
    ec_post_request(
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
        if ec_signed_package_exists("signed/signed.zip", signing_data):
            break
        if time.time() >= end_time:
            raise RuntimeError("Signed package did not appear, check signer logs")
        time.sleep(10)
        i += 1

    info("Signed!")

    info("Downloading...")
    run_cmd(["aws", "s3", "cp", f"s3://{signing_data.bucket_name}/signed/signed.zip", "signed.zip"])
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
    mounting_path = pathlib.Path("/Volumes") / dmg_path.name.replace(".dmg", "")

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
    run_cmd(["cp", "-R", app_path, mounting_path])

    # Unmount the dmg
    run_cmd(["hdiutil", "detach", mounting_path])

    # Convert the dmg to zipped, read only - this is the only type that EC will accept!!
    dmg_path.unlink()
    run_cmd(["hdiutil", "convert", tempdmg_path, "-format", "UDZO", "-o", dmg_path])


def apple_notarize_file(file: pathlib.Path, signing_data: EcSigningData):
    name = file.name
    file_type = file.suffix[1:]

    file_to_notarize = file

    if file_type == "app":
        # check the app is ready to be notarized
        # TODO(grant): remove the check=False if this works
        run_cmd(["syspolicy_check", "notary-submission", file], check=False)

        # We can submit dmg files as is, but we have to zip up app files in a specific way
        file_to_notarize = pathlib.Path(f"{name}.zip")
        run_cmd(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", file, file_to_notarize])

    secrets = get_secretmanager_json(signing_data.notarizing_secret_id)

    run_cmd(
        [
            "xcrun",
            "notarytool",
            "submit",
            file_to_notarize,
            "--team-id",
            APPLE_TEAM_ID,
            "--apple-id",
            secrets["appleId"],
            "--password",
            secrets["appleIdPassword"],
            "--wait",
        ]
    )

    run_cmd(["xcrun", "stapler", "staple", file])

    if file_type == "app":
        # Verify notarization for .app
        run_cmd(["spctl", "-a", "-v", file])
        pathlib.Path(file_to_notarize).unlink()

        # check the file is ready to be distributed
        # TODO(grant): remove the check=False if this works
        run_cmd(["syspolicy_check", "distribution", file], check=False)
    else:
        # Verify notarization for .dmg
        run_cmd(["spctl", "-a", "-t", "open", "--context", "context:primary-signature", "-v", file])


def get_secretmanager_json(secret_id: str):
    info(f"Loading secretmanager value: {secret_id}")
    secret_value = run_cmd_output(["aws", "secretsmanager", "get-secret-value", "--secret-id", secret_id])
    secret_string = json.loads(secret_value)["SecretString"]
    return json.loads(secret_string)


class GpgSigner:
    def __init__(self, gpg_id: str, gpg_secret_key: str, gpg_passphrase: str):
        self.gpg_id = gpg_id
        self.gpg_secret_key = gpg_secret_key
        self.gpg_passphrase = gpg_passphrase

        self.gpg_home = pathlib.Path.home() / ".gnupg-tmp"
        self.gpg_home.mkdir(parents=True, exist_ok=True, mode=0o700)

        # write gpg secret key to file
        self.gpg_secret_key_path = self.gpg_home / "gpg_secret"
        self.gpg_secret_key_path.write_bytes(base64.b64decode(gpg_secret_key))

        self.gpg_passphrase_path = self.gpg_home / "gpg_pass"
        self.gpg_passphrase_path.write_text(gpg_passphrase)

        run_cmd(["gpg", "--version"])

        info("Importing GPG key")
        run_cmd(["gpg", "--list-keys"], env=self.gpg_env())
        run_cmd(
            ["gpg", *self.sign_args(), "--allow-secret-key-import", "--import", self.gpg_secret_key_path],
            env=self.gpg_env(),
        )
        run_cmd(["gpg", "--list-keys"], env=self.gpg_env())

    def gpg_env(self) -> Env:
        return {**os.environ, "GNUPGHOME": self.gpg_home}

    def sign_args(self) -> Args:
        return [
            "--batch",
            "--pinentry-mode",
            "loopback",
            "--no-tty",
            "--yes",
            "--passphrase-file",
            self.gpg_passphrase_path,
        ]

    def sign_file(self, path: pathlib.Path) -> List[pathlib.Path]:
        info(f"Signing {path.name}")
        run_cmd(
            ["gpg", "--detach-sign", *self.sign_args(), "--local-user", self.gpg_id, path],
            env=self.gpg_env(),
        )
        run_cmd(
            ["gpg", "--detach-sign", *self.sign_args(), "--armor", "--local-user", self.gpg_id, path],
            env=self.gpg_env(),
        )
        return [path.with_suffix(f"{path.suffix}.asc"), path.with_suffix(f"{path.suffix}.sig")]

    def clean(self):
        info("Cleaning gpg keys")
        shutil.rmtree(self.gpg_home, ignore_errors=True)


def load_gpg_signer() -> Optional[GpgSigner]:
    pgp_secret_arn = os.getenv("FIG_IO_DESKTOP_PGP_KEY_ARN")
    info(f"FIG_IO_DESKTOP_PGP_KEY_ARN: {pgp_secret_arn}")
    if pgp_secret_arn:
        gpg_secret_json = get_secretmanager_json(pgp_secret_arn)
        gpg_id = gpg_secret_json["gpg_id"]
        gpg_secret_key = gpg_secret_json["gpg_secret_key"]
        gpg_passphrase = gpg_secret_json["gpg_passphrase"]
        return GpgSigner(gpg_id=gpg_id, gpg_secret_key=gpg_secret_key, gpg_passphrase=gpg_passphrase)
    else:
        return None
