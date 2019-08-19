# Firecracker Controller

## Requirements

### `vhost_vsock`

The host needs the vhost\_vsock kernel module loaded:

```bash
$ modprobe vhost_vsock
```

In some cases this might result in conflicts with other vsock modules. It is
safe to unload them for our purposes, but obviously make sure they are not used
by other important software in the system.

### Cgroups

The controller currently relies on a parent cgroup called `firecracker`
existing for the `cpu` and `cpuset` resources. This cgroup should be owned by
the user and/or group of the user running the controller. E.g.:

```bash
$ sudo cgcreate -a alevy:users -g cpu,cpuset:/firecracker
```

(replace `alevy:users` with the appropriate UNIX user and group)

The controller cleans up created cgroups as it kills VMs, but in some cases
cgroups may be left behind (e.g. if the controller process is killed
prematurely). You can always delete all of the firecracker related cgroups
after the fact by recursively deleting the cgroup namespace:

```bash
$ sudo cgdelete -r cpu,cpuset:firecracker/
```
