#!/bin/bash
./create_rootfs.sh images/python-base.ext4 images/mb-json-python.ext4.s microbenchmarks/json
./create_rootfs.sh images/python-base.ext4 images/mb-base64-python.ext4.s microbenchmarks/base64
./create_rootfs.sh images/python-base.ext4 images/mb-primes-python.ext4.s microbenchmarks/primes
./create_rootfs.sh images/python-base.ext4 images/mb-http-python.ext4.s microbenchmarks/http-endpoint
