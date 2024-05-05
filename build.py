#!/usr/bin/env python3
import json
from subprocess import Popen, PIPE, run
import sys

# Build.
with Popen(["cargo", "build", "-r", "--message-format", "json-render-diagnostics"], stdout=PIPE) as proc:
    for line in proc.stdout:
        line = json.loads(line)
        reason = line["reason"]
        if reason == "build-finished":
            if line["success"]:
                break
            else:
                sys.exit(1)
        elif reason == "compiler-artifact":
            if line["target"]["name"] == "kernel-dumper":
                out = line["executable"]

# Create payload.
run(["rustup", "run", "nightly", "objcopy", "-O", "binary", out, "kernel-dumper.bin"], check=True)
