from dataclasses import dataclass
from enum import Enum
from typing import Any, Optional, Mapping

from const import APPLE_TEAM_ID


class EcSigningType(Enum):
    DMG = "dmg"
    APP = "app"
    IME = "ime"


@dataclass
class EmbeddedRequirement:
    path: str
    identifier: str
    signing_args: Optional[Mapping[str, Any]]


def manifest(
    type: str,
    name: str,
    identifier: str,
    entitlements: bool | None = None,
    embedded_requirements: list[EmbeddedRequirement] | None = None,
):
    return {
        "type": type,
        "os": "osx",
        "name": name,
        "outputs": {"label": "macos", "path": name},
        "app": {
            "identifier": identifier,
            "signing_requirements": {
                "certificate_type": "developerIDAppDistribution",
                "app_id_prefix": APPLE_TEAM_ID,
                "signing_args": {"entitlements_path": "SIGNING_METADATA/entitlements.plist"} if entitlements else None,
            },
            "embedded_requirements": {
                req.path: {
                    "identifier": req.identifier,
                    "signing_args": req.signing_args,
                }
                for req in embedded_requirements or []
            },
        },
    }


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
                signing_args={},
            ),
            EmbeddedRequirement(
                path="Contents/MacOS/qterm",
                identifier="com.amazon.qterm",
                signing_args={},
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
