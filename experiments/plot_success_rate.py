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

def completion_rate(experiment):
    complete = experiment["number of completed requests"]
    drop = experiment["drop requests (resource)"]
    total = complete + drop
    return complete / total * 100

snapshot_memories = [int(x["total mem"] / 1024) for x in snapshots]
snapshot_completion_rates = [completion_rate(x) for x in snapshots]

nosnapshot_memories = [int(x["total mem"] / 1024) for x in nosnapshots]
nosnapshot_completion_rates = [completion_rate(x) for x in nosnapshots]

fig = plt.figure()
fig.set_size_inches(8,5)
plt.plot(snapshot_memories, snapshot_completion_rates, label='Snapshot')
plt.plot(nosnapshot_memories, nosnapshot_completion_rates, label='No Snapshot')
plt.xlabel('Memory (GB)')
plt.ylabel('Servicable requests (%)')
plt.title('Utilization')
plt.legend()
plt.savefig("%s/utilization.pdf" % (experiment_dir))
plt.savefig("%s/utilization.png" % (experiment_dir))

