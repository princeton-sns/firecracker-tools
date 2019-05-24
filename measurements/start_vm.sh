#!/bin/bash -e

# Start a firecracker VM with the following parameters
# socket: /tmp/firecracker-sb${VM_ID}.sock
# kernel image: vmlinux from https://github.com/firecracker-microvm/firecracker-demo
# rootfs: xenial.rootfs.ext4 from https://github.com/firecracker-microvm/firecracker-demo
# tap: fc-tap${VM_ID}
# VM Config server options are in: 
#   https://github.com/firecracker-microvm/firecracker/blob/master/api_server/swagger/firecracker.yaml


VM_ID="${1:-0}"
IMAGE="${2}"
FS="${3}"
API_SOCKET="${4}"
LOG="${5}"
METRIC="${6}"

logfile="$PWD/$LOG"
metricsfile="$PWD/$METRIC"

RO_DRIVE="$PWD/$FS"
KERNEL="$PWD/$IMAGE"
#TAP_DEV="fc-tap${VM_ID}"

#API_SOCKET="/tmp/firecracker-sb${VM_ID}.sock"
#logfile="$PWD/output/fc-sb${VM_ID}-log"
#metricsfile="$PWD/output/fc-sb${VM_ID}-metrics"
#metricsfile="/dev/null"

echo "Filesystem: $RO_DRIVE"
echo "Kernel image: $KERNEL"
echo "TAP interface: $TAP_DEV"
echo "API_SOCKET: $API_SOCKET"
echo "Logfile: $logfile"
echo "Metricsfile: $metricsfile"


touch $logfile
touch $metricsfile

# Setup TAP device that uses proxy ARP
#MASK_LONG="255.255.255.252"
#FC_IP="$(printf '169.254.%s.%s' $(((4 * VM_ID + 1) / 256)) $(((4 * VM_ID + 1) % 256)))"
#TAP_IP="$(printf '169.254.%s.%s' $(((4 * VM_ID + 2) / 256)) $(((4 * VM_ID + 2) % 256)))"
#FC_MAC="$(printf '02:FC:00:00:%02X:%02X' $((VM_ID / 256)) $((VM_ID % 256)))"
 
KERNEL_BOOT_ARGS="panic=1 pci=off reboot=k tsc=reliable quiet 8250.nr_uarts=0 ipv6.disable=1"
KERNEL_BOOT_ARGS="${KERNEL_BOOT_ARGS} ip=${FC_IP}::${TAP_IP}:${MASK_LONG}::eth0:off"
#echo "$KERNEL_BOOT_ARGS"

echo "Starting Firecracker VMM"
rm -f "$API_SOCKET"
./firecracker --api-sock "$API_SOCKET" --id "fc-${VM_ID}" --seccomp-level 0 &

# Wait for API server to start
while [ ! -e "$API_SOCKET" ]; do
    echo "FC $VM_ID still not ready..."
    sleep 0.01s
done

echo "API Socket ready"

CURL=(curl --silent --show-error --unix-socket "${API_SOCKET}" -i --header Content-Type:application/json --write-out "HTTP %{http_code}")

curl_put() {
   local URL_PATH="$1"
   local OUTPUT RC
   OUTPUT="$("${CURL[@]}" -X PUT --data @- "http://localhost/${URL_PATH#/}" 2>&1)"
   RC="$?"
   if [ "$RC" -ne 0 ]; then
       echo "Error: curl PUT ${URL_PATH} failed with exit code $RC, output:"
       echo "$OUTPUT"
       return 1
   fi
   # Error if output doesn't end with "HTTP 2xx"
   if [[ "$OUTPUT" != *HTTP\ 2[0-9][0-9] ]]; then
       echo "Error: curl PUT ${URL_PATH} failed with non-2xx HTTP status code, output:"
       echo "$OUTPUT"
       return 1
   fi
}


echo ""
echo ""
echo "configuring logger"
curl_put '/logger' <<EOF
{
  "log_fifo": "$logfile",
  "metrics_fifo": "$metricsfile",
  "level": "Debug",
  "show_level": true,
  "show_log_origin":true
}
EOF

echo ""
echo ""
echo "Get current config"
curl --unix-socket "$API_SOCKET" -i \
     -X GET 'http://localhost/machine-config'   \
     -H 'Accept: application/json'           \
     -H 'Content-Type: application/json'

     
echo "" 
echo "" 
echo "configuring machine resources"
curl_put '/machine-config' <<EOF
{
  "vcpu_count": 1,
  "mem_size_mib": 512
}
EOF

echo ""
echo ""
echo "Check current config"
curl --unix-socket "$API_SOCKET" -i \
     -X GET 'http://localhost/machine-config'   \
     -H 'Accept: application/json'           \
     -H 'Content-Type: application/json'


echo ""
echo ""
echo "Set boot source"
curl_put '/boot-source' <<EOF
{
  "kernel_image_path": "$KERNEL",
  "boot_args": "$KERNEL_BOOT_ARGS"
}
EOF

echo ""
echo ""
echo "setting filesystem"
curl_put '/drives/rootfs' <<EOF
{
      "drive_id": "rootfs",
        "path_on_host": "$RO_DRIVE",
  "is_root_device": true,
  "is_read_only": false 
}
EOF

#echo ""
#echo ""
#echo "configuring virtual network interface"
#curl_put '/network-interfaces/1' <<EOF
#{
#  "iface_id": "1",
#  "guest_mac": "$FC_MAC",
#  "host_dev_name": "$TAP_DEV"
#}
#EOF

echo ""
echo ""
echo "Booting VM..."
curl --unix-socket "$API_SOCKET" -i \
    -X PUT 'http://localhost/actions'       \
    -H  'Accept: application/json'          \
    -H  'Content-Type: application/json'    \
    -d '{
        "action_type": "InstanceStart"
     }'

echo "VM start issued"
