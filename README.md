# About

A sample UEFI program that prints device paths of removable media.

This is a [Rust](https://www.rust-lang.org) port of
[device-path](https://github.com/nkaretnikov/device-path), based on a [post by
Gil Mendes](https://medium.com/@gil0mendes/an-efi-app-a-bit-rusty-82c36b745f49).
(Note that this project uses a more recent version of uefi-rs.)

The `build.py` script is based on the one by Gil Mendes (also see
`uefi-rs/uefi-test-runner/build.py`, which has more features).

# Build and run

In order to test the program with removable media, you need to passthru a USB
drive to QEMU.

## macOS

It doesn't seem to be possible on Mojave with SIP enabled.  Supposedly, you
could work around this in previous macOS versions by unloading the relevant
Apple kernel extension (kext).  It seems the only way to do it now is to develop
a codeless kext, listing devices you want to claim.  But in order to load it,
you would need to request a kext signing certificate (called "Developer ID
Application"), which Apple doesn't give out to individuals, or disable SIP,
which is a security risk.  Otherwise, libusb can't claim the device when QEMU
starts:
```
libusb: error [darwin_claim_interface] USBInterfaceOpen: another process has device opened for exclusive access
libusb: error [darwin_claim_interface] interface not found
```

Links:
* [libusb FAQ](https://github.com/libusb/libusb/wiki/FAQ#How_can_I_run_libusb_applications_under_Mac_OS_X_if_there_is_already_a_kernel_extension_installed_for_the_device)
* [Exclusive access error](https://stackoverflow.com/questions/31699051/usbinterfaceopen-always-report-kioreturnexclusiveaccess-error)
* [Certificate types](https://stackoverflow.com/questions/47231738/kextutil-says-my-kernel-extension-signature-is-invalid-but-code-sign-says-it-is)
* [Codeless kext](https://developer.apple.com/library/archive/technotes/tn2315/_index.html)
* [Apple codeless kext example](https://developer.apple.com/library/archive/technotes/tn2315/tn2315_SampleUSBFTDICodelessKext.zip)
* [Third-party codeless kext example](https://github.com/ilyatikhonov/k8055-mac-codeless-kext).

## Linux

Use `lsusb` to identify the device:
```
Bus xxx Device xxx: ID xxxx:xxxx <Device>
    ^^^        ^^^     ^^^^ ^^^^
    1          2       3    4
```

1. `$HOSTBUS`
2. `$HOSTADDR`
3. `$VENDORID`
4. `$PRODUCTID`


Configure the environment:
```
$ # curl https://sh.rustup.rs -sSf | sh -s -- -y
$ rustup default nightly
$ cargo install cargo-xbuild
$ rustup component add rust-src
```

Build OVMF:
```
$ ./build-ovmf.sh
```

Build:
```
$ git submodule update --init
$ ./build.py build
```

Run (requires QEMU with USB support):
```
$ ./build.py run --ovmf-code=OVMF_CODE.fd --ovmf-vars=OVMF_VARS.fd --hostbus=$HOSTBUS --hostaddr=$HOSTADDR
$ # Or use: --vendorid=$VENDORID --productid=$PRODUCTID
```

Enter UEFI Shell (Boot Manager -> EFI Internal Shell) and confirm whether the
program correctly identifies removable media:
```
Shell> map -v
```

Use `Ctrl-C` to exit QEMU.
