#!/bin/bash

ps aux | grep fire | tr -s ' ' | cut -d ' ' -f 2 | while read -r line; do
    echo $line
    sudo kill -9 $line
done

