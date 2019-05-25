# Create the right rootfs for an app
To create the right rootfs for an app, first create a base rootfs capable
of running python or js. Then from that base rootfs, create app-specific
rootfs using `create_rootfs.sh`.

## Create Python Base rootfs
### create the ext4 file
```
dd if=/dev/zero of=images/python-base.ext4 bs=1M count=700
mkfs.ext4 images/python-base.ext4

sudo umount /tmp/my-rootfs
sudo mount images/python-base.ext4 /tmp/my-rootfs
```

### Start building the alpine linux fs
run under firecracker-tools/measurements
`docker run -it --rm -v /tmp/my-rootfs:/my-rootfs -v $PWD:/ref-rootfs alpine`

### install packages
```
apk update
apk add openrc util-linux python3 python3-dev python2 python2-dev vim bash gcc g++ zip unzip
```
And a couple packages so that our workload can run
```
pip3 install markdown textblob
```

### change root passwd (for debugging only)
`passwd`

### create terminal (for debugging only)
```
ln -s agetty /etc/init.d/agetty.ttyS0
echo ttyS0 > /etc/securetty
rc-update add agetty.ttyS0 default	
rc-update add agetty.ttyS0 nonetwork
```

### copy and register demo-workload startup script
```
cp ref-rootfs/demo-workload /etc/init.d/
rc-update add demo-workload default
```

### Install portio library for python
```
cp -r ref-rootfs/portio-0.5 /
cd portio-0.5
python setup.py install
python3 setup.py install
cd /
```

### Copy the current alpine linux container image to my-rootfs
`for d in bin etc lib root sbin usr home srv; do tar c "$d" | tar x -C my-rootfs; done`

### copy customized /sbin/init
```
rm my-rootfs/sbin/init
cp ref-rootfs/init my-rootfs/sbin
```

### create other necessary directories.
```
bash
mkdir /my-rootfs/{dev,proc,run,sys,tmp,mnt,var}
```

### copying timestamp program `ts` into /srv and compile
If compiled on an non-alpine system, ts doesn't work consistently
```
cp ref-rootfs/ts.c my-rootfs/srv
gcc my-rootfs/srv/ts.c -o my-rootfs/srv/ts
```

### copying workload.sh into /srv
`cp ref-rootfs/workload.sh my-rootfs/srv`

## Create app-specific rootfs
Once you have a base rootfs (that can run python for example), you can start building
app-specific rootfs.

`./create_rootfs.sh <path-to-base-rootfs> <path-to-target-rootfs> <path-to-app>`

It creates a new `ext4` file for the app based on the `base rootfs` and copies the
content of the app directory to `/srv` of the target rootfs.

See `create_workload_rootfs.sh` for examples.

For current testing workloads, adapt `create_workload_rootfs.sh` to create rootfs
for all apps in batch.

# Latency Measurement

`./latency_test.sh <num_vms> <kernel> <rootfs> <network>`

The last option is to specify if the VMs need network or not. Leaving it unspecified
will default to no network.

Running the `latency_test` will output all VMs' logs to `$PWD/output`.
You will see files like `fc-log-0` in that directory.
Once those log files are generated, use `extract_time.sh` to extract latency numbers
for different stages of the function's lifetime.
