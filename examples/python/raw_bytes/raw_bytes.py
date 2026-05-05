#!/usr/bin/env python3
"""
A minimal example that demonstrates using the Raw Bytes view.

Multiple archetypes and components are supported.
"""

from __future__ import annotations

import argparse

import numpy as np

import rerun as rr  # pip install rerun-sdk

DESCRIPTION = """
# Raw Bytes

This is a minimal example that logs synthetic binary data, and then displays it in the rerun viwer.
The underlying data is generated using numpy and visualized using Rerun.

The full source code for this example is available
[on GitHub](https://github.com/rerun-io/rerun/blob/latest/examples/python/raw_bytes/).
""".strip()


def log_data() -> None:
    rr.log(
        "description",
        rr.TextDocument(DESCRIPTION, media_type=rr.MediaType.MARKDOWN),
        static=True,
    )

    rr.log(
        "some/random/bytes",
        rr.RawBytes(rr.Blob(np.random.bytes(128))),
    )
    rr.log(
        "not-really-json",
        rr.RawBytes(blob=rr.Blob(b"{hello: this, is: {some: text, 2: 3}}")),
    )
    rr.log(
        "password/symbols",
        rr.RawBytes(b"HMACG%v*MAnx$nVf4cj1jUtw3esNqgfr7aWuB%KZF21yT^KyncvPp8vHb%ngNvbV"),
    )
    rr.log(
        "password/memorable",
        rr.RawBytes(b"Carbon1*Curling52Sixth1^Clapping04Crewless3"),
    )
    rr.log(
        "text",
        rr.TextDocument("Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos."),
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

