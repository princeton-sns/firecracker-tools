#!/usr/bin/env python3

import json
import glob
import os
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

experiment_dir = sys.argv[1]

datas = [json.load(open(f, 'r')) for f in glob.glob(experiment_dir + "/results/*.json")]

snapshots = [x for x in datas if x["snapshot"]]
nosnapshots = [x for x in datas if x["snapshot"] != True]

snapshots.sort(key=lambda e: e["total mem"])
nosnapshots.sort(key=lambda e: e["total mem"])

def throughput(experiment):
    return experiment["cumulative throughput"]

snapshot_memories = [int(x["total mem"] / 1024) for x in snapshots]
snapshot_throughput = [throughput(x) for x in snapshots]

nosnapshot_memories = [int(x["total mem"] / 1024) for x in nosnapshots]
nosnapshot_throughput = [throughput(x) for x in nosnapshots]

fig = plt.figure()
fig.set_size_inches(8,5)
plt.plot(snapshot_memories, snapshot_throughput, label='Snapshot')
plt.plot(nosnapshot_memories, nosnapshot_throughput, label='No Snapshot')
plt.xlabel('Memory (GB)')
plt.ylabel('Avg. Throughput (req/s)')
plt.title('Throughput vs. Memory')
plt.legend()
plt.savefig("%s/throughput.pdf" % (experiment_dir))
plt.savefig("%s/throughput.png" % (experiment_dir))

