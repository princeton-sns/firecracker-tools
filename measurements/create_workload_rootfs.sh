#!/bin/bash
BASE="${1}"
PREFIX="${2}"

./create_rootfs.sh $BASE images/json-python.ext4 workload/microbenchmarks/json
./create_rootfs.sh $BASE images/base64-python.ext4 workload/microbenchmarks/base64
./create_rootfs.sh $BASE images/primes-python.ext4 workload/microbenchmarks/primes
./create_rootfs.sh $BASE images/http-python.ext4 workload/microbenchmarks/http-endpoint

./create_rootfs.sh $BASE images/markdown2html-python.ext4 workload/markdown-to-html
./create_rootfs.sh $BASE images/sentiment-python.ext4 workload/sentiment-analysis

