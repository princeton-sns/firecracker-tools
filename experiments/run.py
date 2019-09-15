#!/usr/bin/env python3

import os
import sys
import time
import math

experiments_dir = os.path.dirname(sys.argv[0])

experiment_options = next(os.walk(experiments_dir))[1]

print(experiment_options)
experiment = input("Which experiment do you want to run? ")
min_memory = input("What memory do you want to start with (GBs)? ")
max_memory = input("What memory do you want to end with (GBs)? ")

min_memory = int(min_memory)
max_memory = int(max_memory)

print("OK, running experiment \"%s\" from %dMB to %dMB in 1GB increments" % (experiment, min_memory, max_memory))

os.chdir(experiments_dir)

increment_base = 128
runs = 30

increment = math.ceil((max_memory * 1024 - min_memory * 1024) / increment_base / runs) * increment_base

memory = min_memory * 1024
count = 0
total = math.ceil((max_memory - min_memory) * 1024 / increment)
while memory <= (max_memory * 1024):
    count += 1
    print("Run %d of %d" % (count, total)) 
    print("\t+ Running snapshot version...")
    os.system("make MEMSIZE=%d MODE=snapshot EXPERIMENT=%s run > /dev/null" % (memory, experiment))
    print("\t+ Running non-snapshot version...")
    os.system("make MEMSIZE=%d MODE=nosnapshot EXPERIMENT=%s run > /dev/null" % (memory, experiment))
    memory += increment
    time.sleep(20)
