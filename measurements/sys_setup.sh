#!/bin/bash

# Configure the TAP network interface for COUNT many microVMs
# The number of microVMs should be passed in as the first argument

COUNT="${1:-5}"
NET="${2:-nonet}"

# /output is for all VM logs
rm -rf output
mkdir output
chown -R $USER output

# Load kernel module
sudo modprobe kvm_intel

if [ $NET != "nonet" ]; then
    # Configure packet forwarding
    echo "Setting up $COUNT TAP devices"
    sudo sysctl -w net.ipv4.conf.all.forwarding=1

    # Avoid "neighbour: arp_cache: neighbor table overflow!"
    sudo sysctl -w net.ipv4.neigh.default.gc_thresh1=1024
    sudo sysctl -w net.ipv4.neigh.default.gc_thresh2=2048
    sudo sysctl -w net.ipv4.neigh.default.gc_thresh3=4096

    for ((i=0; i<COUNT; i++))
    do
        ./network_tap_setup.sh $i
    done
fi
