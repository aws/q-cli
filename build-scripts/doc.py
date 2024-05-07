from const import DESKTOP_PACKAGE_NAME
from util import isBrazil, isLinux, run_cmd


def run_doc():
    doc_args = ["cargo", "doc", "--no-deps", "--workspace"]
    if isLinux():
        doc_args.extend(["--exclude", DESKTOP_PACKAGE_NAME])

    run_cmd(doc_args)

    if isBrazil():
        run_cmd(["cargo", "brazil", "export-docs"])
