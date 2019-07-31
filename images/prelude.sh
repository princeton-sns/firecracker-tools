apk add openrc util-linux

## Create start script for that mounts the appfs and invokes whatever binary is in /srv/workload
printf '#!/bin/sh\n
mount /dev/vdb /srv\n
exec /srv/workload\n' > /bin/workload
chmod +x /bin/workload

## Have the start script invoked by openrc/init
printf '#!/sbin/openrc-run\n
command="/bin/workload"\n' > /etc/init.d/serverless-workload
chmod +x /etc/init.d/serverless-workload
rc-update add serverless-workload default

## Setup console
ln -s agetty /etc/init.d/agetty.ttyS0
echo ttyS0 > /etc/securetty
rc-update add agetty.ttyS0 default
rc-update add agetty.ttyS0 nonetwork

echo agetty_options=\"-a root\" >> /etc/conf.d/agetty

## Add /dev and /proc file systems to openrc's boot
rc-update add devfs boot
rc-update add procfs boot
