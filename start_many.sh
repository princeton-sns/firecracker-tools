#!/bin/bash

start="${1:-0}"
end="${2:-1}"

for (( i=start; i<end; i++ ))
do
    ./start_vm.sh "$i"
done
