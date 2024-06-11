from dataclasses import dataclass
from enum import Enum

from const import APPLE_TEAM_ID


class EcSigningType(Enum):
    DMG = "dmg"
    APP = "app"
    IME = "ime"


@dataclass
class EmbeddedRequirement:
    path: str
    identifier: str


def manifest(
    type: str,
    name: str,
    identifier: str,
    entitlements: bool | None = None,
    embedded_requirements: list[EmbeddedRequirement] | None = None,
):
    m = {
        "type": type,
        "os": "osx",
        "name": name,
        "outputs": [{"label": "macos", "path": name}],
        "app": {
            "identifier": identifier,
            "signing_requirements": {
                "certificate_type": "developerIDAppDistribution",
                "app_id_prefix": APPLE_TEAM_ID,
            },
        },
    }

    if entitlements:
        m["app"]["signing_args"] = {"entitlements_path": "SIGNING_METADATA/entitlements.plist"}

    if embedded_requirements:
        m["app"]["embedded_binaries"] = {
            req.path: {
                "identifier": req.identifier,
            }
            for req in embedded_requirements
        }

    return m


def app_manifest():
    return manifest(
        type="app",
        name="Amazon Q.app",
        identifier="com.amazon.codewhisperer",
        entitlements=True,
        embedded_requirements=[
            EmbeddedRequirement(
                path="Contents/MacOS/q",
                identifier="com.amazon.q",
            ),
            EmbeddedRequirement(
                path="Contents/MacOS/qterm",
                identifier="com.amazon.qterm",
            ),
        ],
    )


def dmg_manifest(name: str):
    return manifest(
        type="dmg",
        name=name,
        identifier="com.amazon.codewhisperer.installer",
    )


def ime_manifest():
    return manifest(
        type="app",
        name="CodeWhispererInputMethod.app",
        identifier="com.amazon.inputmethod.codewhisperer",
    )
