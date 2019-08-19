# Getting Started with Firerunner
## Prerequisites
### Firecracker
First make sure your system is able to run [Firecracker VMM](https://github.com/firecracker-microvm/firecracker/blob/master/docs/getting-started.md). You can use the following script to check system readiness:

```bash
err="";
    [ "$(uname) $(uname -m)" = "Linux x86_64" ]  \
      || [ "$(uname) $(uname -m)" = "Linux aarch64" ] \
      || err="ERROR: your system is not Linux x86_64 or Linux aarch64."; \
    [ -r /dev/kvm ] && [ -w /dev/kvm ] \
      || err="$err\nERROR: /dev/kvm is innaccessible."; \
    (( $(uname -r | cut -d. -f1)*1000 + $(uname -r | cut -d. -f2) >= 4014 )) \
      || err="$err\nERROR: your kernel version ($(uname -r)) is too old."; \
    dmesg | grep -i "hypervisor detected" \
      && echo "WARNING: you are running in a virtual machine." \
      && echo "Firecracker is not well tested under nested virtualization."; \
    [ -z "$err" ] && echo "Your system looks ready for Firecracker!" || echo -e "$err"
```

Sometimes after restart `/dev/kvm` becomes inaccessible. Grant your user access with the ACL utility:

```bash
$ sudo setfacl -m u:${USER}:rw /dev/kvm
```

### Vsock

Currently, Firerunner uses vsock for passing input from host to guest VM and returning output from guest VM to host. Therefore the host needs the `vhost_vsock` kernel module loaded.

Check if vsock kernel module is already loaded with:

`$ lsmod | grep vsock`

If not, load it with `$ sudo modprobe vhost_vsock`

In some cases this might result in conflicts with other vsock modules. It is safe to unload them for our purposes, but obviously make sure they are not used by other important software in the system.

Also add your user to vsock's ACL:

`$ sudo setfacl -m u:${USER}:rw /dev/vhost-vsock`

### Cgroups

The controller currently relies on a parent cgroup called `firecracker` existing for the `cpu`and `cpuset` resources. This cgroup should be owned by the user and/or group of the user running the controller. E.g.:

```bash
$ sudo cgcreate -a alevy:users -g cpu,cpuset:/firecracker
```

(replace `alevy:users` with the appropriate UNIX user and group)

`cgcreate` is part of the `cgroup-tools` suite. Make sure it is installed.

The controller cleans up created cgroups as it kills VMs, but in some cases cgroups may be left behind (e.g. if the controller process is killed prematurely). You can always delete all of the firecracker related cgroups after the fact by recursively deleting the cgroup namespace:

```bash
$ sudo cgdelete -r cpu,cpuset:firecracker/
```
