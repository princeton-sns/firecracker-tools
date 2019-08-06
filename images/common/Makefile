mkfile_path := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

all: nc-vsock

nc-vsock: nc-vsock.c
	echo "apk add alpine-sdk linux-headers; gcc -o /common/nc-vsock /common/nc-vsock.c" | docker run -i --rm -v $(mkfile_path):/common alpine
