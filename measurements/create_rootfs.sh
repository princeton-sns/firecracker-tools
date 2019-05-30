#!/bin/bash
base="${1}"
target="${2}"
app="${3}"

echo "creating $target based on $base"
mkdir -p /tmp/my-rootfs
sudo umount /tmp/my-rootfs
cp $base $target
sudo mount $target /tmp/my-rootfs

echo "copying app: $app"
#sudo cp workload/$app/app.alpine.zip /tmp/my-rootfs/srv/
sudo cp -r $PWD/$app/* /tmp/my-rootfs/srv

# Copying binary compiled on Ubuntu to a Alpine rootfs doesn't work
#sudo cp ts /tmp/my-rootfs/srv/

