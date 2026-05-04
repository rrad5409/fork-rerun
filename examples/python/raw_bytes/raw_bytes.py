#!/usr/bin/env python3
"""
An 
"""

from __future__ import annotations

import argparse
from math import tau

import numpy as np

import rerun as rr  # pip install rerun-sdk
from rerun import blueprint as rrb

DESCRIPTION = """
# Raw Bytes

This is a minimal example that logs synthetic binary data, and then displays it in the rerun viwer.
The underlying data is generated using numpy and visualized using Rerun.

The full source code for this example is available
[on GitHub](https://github.com/rerun-io/rerun/blob/latest/examples/python/raw_bytes/).
""".strip()


def log_data() -> None:
    rr.log("description", rr.TextDocument(DESCRIPTION, media_type=rr.MediaType.MARKDOWN), static=True)

    rr.set_time("log_time", duration=0)

    by = b"{hello: this, is: {some: text, 2: 3}}" + np.random.bytes(128)

    rr.log(
        "helix/structure/scaffolding",
        rr.RawBytes(
            blob=rr.Blob(by)
        ),
        static=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    rr.script_add_args(parser)
    args = parser.parse_args()

    rr.script_setup(args, "rerun_example_raw_bytes")
    log_data()

    rr.script_teardown(args)


if __name__ == "__main__":
    main()

