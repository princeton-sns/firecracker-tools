import numpy as np
from enum import Enum
import yaml
import operator
import functools
import random
import sys
import json


def finished(si, ni):
    for i in si:
        if i < ni - 1:
            return False
    return True


workload_config_file = sys.argv[1]
output_request_file = sys.argv[2]
print('loading workload config from: ' + workload_config_file)

with open(workload_config_file) as f:
    config = f.read()

data = yaml.load(config, Loader=yaml.Loader) # a list of dicts

function_names = [f['name'] for f in data]
arrival_rates = np.array([f['arrival_rate'] for f in data]) # num of invocations per second
num_invocations = np.array([f['num_invocation'] for f in data])

print('function names: ' + str(function_names))
print('arrival rates: ' + str(arrival_rates))
print('number of invocations: ' + str(num_invocations))


mu = 1/arrival_rates
num_functions = len(arrival_rates)
num_invocations = num_invocations[0]
inter_arrival_time = np.random.exponential(mu, (num_invocations, num_functions))
inter_arrival_time = inter_arrival_time * 1000 # convert to ms
inter_arrival_time = np.ceil(inter_arrival_time)
inter_arrival_time_cumsum = np.cumsum(inter_arrival_time, axis=0)

search_index = np.zeros(num_functions, dtype=np.int32)

inter_arrival_time_cumsum = inter_arrival_time_cumsum.astype(int)
#print(inter_arrival_time_cumsum )

fd = open(output_request_file, 'w')

pmin = 0

while not finished(search_index, num_invocations):
    candidates = [inter_arrival_time_cumsum[search_index[i], i] for i in range(num_functions)]
    minv = np.min(candidates)
    min_idx = np.argmin(candidates)

    interval = minv - pmin
    pmin = minv

    json.dump({"interval": int(interval), "function": function_names[min_idx], "payload":{"request": 42}}, fd)
    fd.write('\n')

    search_index[min_idx] = search_index[min_idx] + 1

    if search_index[min_idx] == num_invocations:
        inter_arrival_time_cumsum[num_invocations - 1, min_idx] = sys.maxsize
        search_index[min_idx] = search_index[min_idx] - 1


candidates = [inter_arrival_time_cumsum[search_index[i], i] for i in range(num_functions)]
minv = np.min(candidates)
min_idx = np.argmin(candidates)

interval = minv - pmin

json.dump({"interval": int(interval), "function": function_names[min_idx], "payload":{"request": 42}}, fd)

fd.close()
