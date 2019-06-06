#!/bin/bash

mkdir -p output
cd ../firerunner; cargo build --release
cd ../measurements

echo "Start memory usage test @ `date`"

for app in base64 http json sentiment markdown2html primes
do
>logs
sudo setfacl -m u:$(id -un):rw /dev/kvm
../firerunner/target/release/firerunner \
	--kernel images/hello-vmlinux.bin \
	--rootfs images/$app-python.ext4
mv pages.log output/$app-pages.log
done
