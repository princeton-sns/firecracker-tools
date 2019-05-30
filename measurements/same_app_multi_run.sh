#!/bin/bash

if [ $# -ne 2 ]; then
	echo 'usage: ./same_app_multi_run.sh app-name number-of-runs'
	exit 1
fi

APP=$1
RUN=$2

mkdir -p mem_dump

for ((i=1;i<=$RUN;i++))
do
	../firerunner/target/release/firerunner \
		--kernel ../firerunner/hello-vmlinux.bin \
		--rootfs images/$APP-python.ext4
	mv boot_mem_dump mem_dump/boot_$i
	mv init_mem_dump mem_dump/init_$i
	mv python_mem_dump mem_dump/python_$i
done
