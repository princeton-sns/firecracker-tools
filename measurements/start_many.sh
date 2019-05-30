#!/bin/bash

start="${1:-0}"
end="${2:-1}"
KERNEL="${3}"
FS="${4}"

for (( i=start; i<end; i++ ))
do
	API_SOCKET="/tmp/fc-sb${i}.sock"
	LOG="output/fc-log-${i}"
	METRIC="output/fc-sb${i}-metrics"
    ./start_vm.sh "$i" $KERNEL $FS $API_SOCKET $LOG $METRIC
	sleep 0.5

done
