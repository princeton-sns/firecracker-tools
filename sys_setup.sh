#!/bin/bash

# Configure the TAP network interface for COUNT many microVMs
# The number of microVMs should be passed in as the first argument

COUNT="${1:-5}"
echo "Setting up $COUNT TAP devices"

# /output is for all VM logs
rm -rf output
mkdir output
chown -R $USER output

# Load kernel module
sudo modprobe kvm_intel

# Configure packet forwarding
sudo sysctl -w net.ipv4.conf.all.forwarding=1

# Avoid "neighbour: arp_cache: neighbor table overflow!"
sudo sysctl -w net.ipv4.neigh.default.gc_thresh1=1024
sudo sysctl -w net.ipv4.neigh.default.gc_thresh2=2048
sudo sysctl -w net.ipv4.neigh.default.gc_thresh3=4096

for ((i=0; i<COUNT; i++))
do
    ./network_tap_setup.sh $i
done
