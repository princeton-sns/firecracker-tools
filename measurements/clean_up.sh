#!/bin/bash
# Delete TAP devices

COUNT="${1:-5}" # Default to 0

for (( i=0; i<COUNT; i++ ))
do
    echo $i
    ip link del "fc-tap${i}" 2> /dev/null
done

