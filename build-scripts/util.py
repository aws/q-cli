from enum import Enum
import os
import shlex
import subprocess
import platform
from typing import Mapping, Sequence


INFO = "\033[95m"
FAIL = "\033[91m"
ENDC = "\033[0m"


def isCi() -> bool:
    return os.environ.get("CI") is not None


def isDarwin() -> bool:
    return platform.system() == "Darwin"


def isLinux() -> bool:
    return platform.system() == "Linux"


def info(*s: str):
    if isCi():
        print(f"INFO: {' '.join(s)}")
    else:
        print(f"{INFO}INFO:{ENDC} {' '.join(s)}")


def fail(*s: str):
    if isCi():
        print(f"FAIL: {' '.join(s)}")
    else:
        print(f"{FAIL}FAIL:{ENDC} {' '.join(s)}")


Args = Sequence[str | os.PathLike]
Env = Mapping[str, str | os.PathLike]
Cwd = str | os.PathLike


def run_cmd(args: Args, env: Env | None = None, cwd: Cwd | None = None, check: bool = True):
    args_str = [str(arg) for arg in args]
    print(f"+ {shlex.join(args_str)}")
    subprocess.run(args, env=env, cwd=cwd, check=check)


def run_cmd_output(
    args: Args,
    env: Env | None = None,
    cwd: Cwd | None = None,
) -> str:
    res = subprocess.run(args, env=env, cwd=cwd, check=True, stdout=subprocess.PIPE)
    return res.stdout.decode("utf-8")


def run_cmd_status(
    args: Args,
    env: Env | None = None,
    cwd: Cwd | None = None,
) -> int:
    res = subprocess.run(args, env=env, cwd=cwd)
    return res.returncode


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
