#!/bin/bash

#DATA_DIR=${1:-.}
DATA_DIR="output"
KERNEL="$PWD/kernel.log"
INIT="$PWD/init.log"
PY="$PWD/python.log"
IMPORT="$PWD/import.log"
APP="$PWD/app.log"
FC_SETUP="$PWD/fc_setup.log"

rm -f $KERNEL $INIT $IMPORT $APP $FC_SETUP $PY

pushd $DATA_DIR > /dev/null

COUNT=`ls fc-log* | sort -V | tail -1 | cut -d '-' -f 3 | cut -f 2 -d 'b'`
echo "$COUNT VM instances"

for i in `seq 0 $COUNT`
do
    fc_setup_time=`grep FC-guest fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ':' | cut -f 2 -d ' '`
    kernel_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==1'`
    init_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==2'`
    python_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==3'`
    import_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==4'`
    app_time=`grep Guest-boot fc-log-${i} | cut -f 2 -d '=' |cut -f 2 -d ',' |  cut -f 5 -d ' ' | awk 'NR==5'`
	echo "$fc_setup_time" >>$FC_SETUP
    echo "$kernel_time" >> $KERNEL
    echo "$init_time" >> $INIT
	echo "$python_time" >> $PY
	echo "$import_time" >> $IMPORT
    echo "$app_time" >> $APP
done

popd > /dev/null
