import os
import shlex
import subprocess
import sys
from typing import Any, Mapping, Sequence


IS_DARWIN = sys.platform.startswith("darwin")
IS_LINUX = sys.platform.startswith("linux")

INFO = "\033[95m"
FAIL = "\033[91m"
ENDC = "\033[0m"


def info(s: str):
    print(f"{INFO}INFO:{ENDC} {s}")


def fail(s: str):
    print(f"{FAIL}FAIL:{ENDC} {s}")


Args = Sequence[str | os.PathLike]
Env = Mapping[str, str | os.PathLike]
Cwd = str | os.PathLike


def run_cmd(
    args: Args,
    env: Env | None = None,
    cwd: Cwd | None = None,
):
    args_str = [str(arg) for arg in args]
    print(f"+ {shlex.join(args_str)}")
    subprocess.run(args, env=env, cwd=cwd, check=True)


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


# Emulating -n from bash
def n(val: Any, key: str) -> bool:
    if key in val:
        if isinstance(val[key], str) and len(val[key]) > 0:
            return True
        elif isinstance(val[key], int):
            return True
        elif isinstance(val[key], bool):
            return True
        else:
            return False
    else:
        return False
