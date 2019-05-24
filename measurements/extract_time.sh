#!/bin/bash

#DATA_DIR=${1:-.}
DATA_DIR="output"
DEST="$PWD/data.log"
KERNEL="$PWD/kernel.log"
INIT="$PWD/init.log"
UNZIP="$PWD/unzip.log"
APP="$PWD/python.log"

rm -f $DEST

pushd $DATA_DIR > /dev/null

COUNT=`ls fc-log* | sort -V | tail -1 | cut -d '-' -f 3 | cut -f 2 -d 'b'`

for i in `seq 0 $COUNT`
do
    kernel_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==1'`
    init_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==2'`
    unzip_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==3'`
    app_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==4'`
    #echo "$i boot $boot_time ms" >> $DEST
    echo "$kernel_time" >> $KERNEL
    echo "$init_time" >> $INIT
	echo "$unzip_time" >> $UNZIP
    echo "$app_time" >> $APP
done

popd > /dev/null
