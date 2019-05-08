#!/bin/bash

#DATA_DIR=${1:-.}
DATA_DIR="output"
DEST="$PWD/data.log"
KERNEL="$PWD/kernel.log"
INIT="$PWD/init.log"
PY="$PWD/python.log"

rm -f $DEST

pushd $DATA_DIR > /dev/null

COUNT=`ls fc-sb* | sort -V | tail -1 | cut -d '-' -f 2 | cut -f 2 -d 'b'`

for i in `seq 0 $COUNT`
do
    kernel_time=`grep Guest-boot fc-sb${i}-log | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==1'`
    init_time=`grep Guest-boot fc-sb${i}-log | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==2'`
    python_time=`grep Guest-boot fc-sb${i}-log | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==3'`
    #echo "$i boot $boot_time ms" >> $DEST
    echo "$kernel_time" >> $KERNEL
    echo "$init_time" >> $INIT
    echo "$python_time" >> $PY
done

popd > /dev/null
