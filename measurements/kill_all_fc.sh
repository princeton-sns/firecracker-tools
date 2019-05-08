#!/bin/bash

ps aux | grep firecracker | tr -s ' ' | cut -d ' ' -f 2 | while read -r line; do
    echo $line
    sudo kill -9 $line
done

