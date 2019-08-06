#!/usr/bin/env bash

coproc /usr/bin/nc-vsock 0 1234

python2 /bin/runtime-workload.py <&${COPROC[0]} 2>&${COPROC[1]}
