# Firecracker tools

## Firecracker VM config and launch example
Take a look at `start_vm.sh`. For the bare minimum, you need to specify the kernel
image and a rootfs. Common additional configs include logger, machine specs, and
network.
For a complete list of config options,
see [Firecracker API server's yaml file](https://github.com/firecracker-microvm/firecracker/blob/master/api_server/swagger/firecracker.yaml)

## Network setup
TAP devices are created and configured ahead of time. See `sys_setup.sh` and `network_tap_setup.sh`
for examples.

## Example rootfs
I've included the following rootfs as examples since making the right rootfs has proven to be tricky for me:
1. hello-rootfs.ext4: provided by AWS team as a toy example. Actually not a minimum rootfs. init
   process does quite a bit
2. iperf.rootfs.ext4: similar to hello-rootfs but added a network job (iperf) to the init process.
3. lt-rootfs.ext4: used for cold launch latency tests. Has a C program and a Python program
   at the end of init process writing to a magic port for time measurement.

## How to tailor make a rootfs for your function
TODO
