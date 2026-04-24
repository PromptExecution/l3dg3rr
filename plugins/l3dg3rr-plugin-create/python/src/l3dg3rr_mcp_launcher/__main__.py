from __future__ import annotations

import argparse
import os
import subprocess
import sys


def build_command(args: argparse.Namespace) -> list[str]:
    if args.mode == "cargo":
        return ["cargo", "run", "-p", "ledgerr-mcp", "--bin", "ledgerr-mcp-server"]
    if args.mode == "binary":
        return [args.binary]
    if args.mode == "docker":
        return [
            "docker",
            "run",
            "-i",
            "--rm",
            "-v",
            f"{os.getcwd()}:/workspace",
            "-w",
            "/workspace",
            args.image,
            "cargo",
            "run",
            "-p",
            "ledgerr-mcp",
            "--bin",
            "ledgerr-mcp-server",
        ]
    raise ValueError(f"unsupported mode: {args.mode}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Launch l3dg3rr ledgerr-mcp server")
    parser.add_argument(
        "--mode",
        choices=["cargo", "binary", "docker"],
        default="cargo",
        help="Runtime profile to launch",
    )
    parser.add_argument(
        "--binary",
        default="./target/release/ledgerr-mcp-server",
        help="Path to compiled server binary when --mode binary",
    )
    parser.add_argument(
        "--image",
        default="tax-ledger:dev",
        help="Docker image to use when --mode docker",
    )
    args = parser.parse_args()

    cmd = build_command(args)
    completed = subprocess.run(cmd, check=False)
    sys.exit(completed.returncode)


if __name__ == "__main__":
    main()
