#!/bin/bash
#
#
COUNT="${1:-5}"
KERNEL="${2}"
FS="${3}"
NET="${4:-nonet}"

# initialze system (network interfaces)
./sys_setup.sh $COUNT $NET

echo "Start latency ($COUNT)test @ `date`"
START_TS=`date +%s%N | cut -b1-13`

./start_many.sh 0 $COUNT $KERNEL $FS&
pids[${i}]=$!
echo PID $pids

# wait for all pids
for pid in ${pids[*]}; do
    wait $pid
done

END_TS=`date +%s%N | cut -b1-13`
END_DATE=`date`

total=$COUNT
delta_ms=$((END_TS-START_TS))
delta=$((delta_ms/1000))
rate=`bc -l <<< "$total/$delta"`

cat << EOL
Done @ $END_DATE.
Started $total microVMs in $delta_ms milliseconds.
EOL

