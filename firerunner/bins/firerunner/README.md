# Firerunner

The `firerunner` utility runs a single Firecracker VM based on a kernel, root
file system, and optional application-specific file system. It also delivers
requests to the guest VM using a VSOCK socket and outputs responses from the
VM.

## Usage

```bash
USAGE:
    firerunner [OPTIONS] --kernel <KERNEL> --rootfs <ROOTFS>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --appfs <APPFS>                    Path to the root file system
    -c, --cmd_line <CMD_LINE>              Command line to pass to the kernel [default: quiet console=none reboot=k
                                           panic=1 pci=off]
        --id <id>                          MicroVM unique identifier [default: abcde1234]
    -k, --kernel <KERNEL>                  Path the the kernel binary
        --rootfs <ROOTFS>                  Path to the root file system
        --seccomp-level <seccomp-level>    Level of seccomp filtering.
                                                   - Level 0: No filtering.
                       
                                                   - Level 1: Seccomp filtering by syscall number.
                       
                                                   - Level 2: Seccomp filtering by syscall number and argument values.
                       
                                                [default: 0]  [possible values: 0, 1, 2]
```

Once launched, `firerunner` reads requests intended for the VM as line-delimited strings from standard in. These need not follow any particular format, but typically will be single-line JSON strings.

Each response from the VM is output to standard out as-is (i.e. they may be multi-line if the VM outputs a response with a newline character).

## Guest VM semantics

To recieve requests from the VMM, the guest VM should connect to the VMM's vsock with channel id VMADDR\_CID\_ANY on port 1234.

Once a connection is established, the VMM will loop on the following two steps:

  1. Send a request

  2. Wait for a response from the guest VM

Each message, both requests and responses, begin with a single length byte followed by that many bytes.

