#!/bin/bash
./create_rootfs.sh images/python-base.ext4 images/mb-json-python.ext4 microbenchmarks/json
./create_rootfs.sh images/python-base.ext4 images/mb-base64-python.ext4 microbenchmarks/base64
./create_rootfs.sh images/python-base.ext4 images/mb-primes-python.ext4 microbenchmarks/primes
./create_rootfs.sh images/python-base.ext4 images/mb-http-python.ext4 microbenchmarks/http-endpoint

./create_rootfs.sh images/python-base.ext4 images/mb-markdown2html-python.ext4 markdown-to-html
