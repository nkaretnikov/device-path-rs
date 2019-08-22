#!/usr/bin/env python3

# Based on
# https://medium.com/@gil0mendes/an-efi-app-a-bit-rusty-82c36b745f49

import argparse
import os
import shutil
import sys
import subprocess as sp
from pathlib import Path

ARCH = "x86_64"
TARGET = ARCH + "-unknown-uefi"
CONFIG = "debug"
QEMU = "qemu-system-" + ARCH

WORKSPACE_DIR = Path(__file__).resolve().parents[0]
BUILD_DIR = WORKSPACE_DIR / "build"
CARGO_BUILD_DIR = WORKSPACE_DIR / "target" / TARGET / CONFIG

APP_NAME = "device-path-rs"

def run_xbuild(*flags):
  "Run Cargo XBuild with the given arguments."

  cmd = ["cargo", "xbuild", "--target", TARGET, *flags]
  sp.run(cmd).check_returncode()

def build_command():
  "Build the UEFI application."

  run_xbuild("--package", APP_NAME)

  # Create the build folder.
  boot_dir = BUILD_DIR / "EFI" / "BOOT"
  boot_dir.mkdir(parents=True, exist_ok=True)

  # Copy the UEFI application to the build directory.
  built_file = CARGO_BUILD_DIR / Path(str(APP_NAME) + ".efi")
  output_file = boot_dir / "BootX64.efi"
  shutil.copy2(built_file, output_file)

  # Write a startup script to make UEFI Shell load the application
  # automatically.
  startup_file = open(BUILD_DIR / "startup.nsh", "w")
  startup_file.write("\EFI\BOOT\BOOTX64.EFI")
  startup_file.close()

def run_command(ovmf_code, ovmf_vars, hostbus, hostaddr, vendorid, productid):
  "Run the application in QEMU."

  qemu_flags = [
    "-nographic",

    # Disable default devices.
    # QEMU by default enables a ton of devices which slow down boot.
    "-nodefaults",

    # Use a standard VGA for graphics.
    "-vga", "std",

    # Use a modern machine, with acceleration if possible.
    "-machine", "q35,accel=kvm:tcg",

    # Allocate some memory.
    "-m", "128M",

    # Set up OVMF.
    "-drive", f"if=pflash,format=raw,readonly,file={ovmf_code}",
    "-drive", f"if=pflash,format=raw,file={ovmf_vars}",

    # Mount a local directory as a FAT partition.
    "-drive", f"format=raw,file=fat:rw:{BUILD_DIR}",

    # Enable serial.
    #
    # Connect the serial port to the host.  OVMF is kind enough to connect
    # the UEFI stdout and stdin to that port too.
    "-serial", "stdio",

    # Setup monitor.
    "-monitor", "vc:1024x768",
  ]

  # Attach the USB drive to the controller.
  if hostbus and hostaddr:
    qemu_flags += [
      "-device", "qemu-xhci,id=xhci",
      "-device", f"usb-host,bus=xhci.0,hostbus={hostbus},hostaddr={hostaddr}",
    ]

  elif vendorid and productid:
    qemu_flags += [
      "-device", "qemu-xhci,id=xhci",
      "-device", f"usb-host,bus=xhci.0,vendorid={vendorid},productid={productid}",
    ]

  sp.run([QEMU] + qemu_flags).check_returncode()

def main(args):
  "Run user-requested actions."

  # Clear any Rust flags which might affect the build.
  os.environ["RUSTFLAGS"] = ""
  os.environ["RUST_TARGET_PATH"] = str(WORKSPACE_DIR)

  usage = "%(prog)s verb [options]"
  desc = "Build script for the UEFI app"

  parser = argparse.ArgumentParser(usage=usage, description=desc)

  subparsers = parser.add_subparsers(dest="verb")
  build_parser = subparsers.add_parser("build")
  run_parser = subparsers.add_parser("run")

  run_parser.add_argument('--ovmf-code', required=True)
  run_parser.add_argument('--ovmf-vars', required=True)

  run_parser.add_argument('--hostbus')
  run_parser.add_argument('--hostaddr')

  run_parser.add_argument('--vendorid')
  run_parser.add_argument('--productid')

  opts = parser.parse_args()

  if opts.verb == "build":
    build_command()

  elif opts.verb == "run":
    run_command(
      ovmf_code=opts.ovmf_code,
      ovmf_vars=opts.ovmf_vars,
      hostbus=opts.hostbus,
      hostaddr=opts.hostaddr,
      vendorid=opts.vendorid,
      productid=opts.productid)

  else:
    print(f"Unknown verb: '{opts.verb}'")

if __name__ == '__main__':
  sys.exit(main(sys.argv))
