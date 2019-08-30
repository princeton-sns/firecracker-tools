import numpy as np
from enum import Enum
import yaml
import operator
import functools
import random
import sys
import json

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
num_invocations = 100
inter_arrival_time = np.random.exponential(mu, (num_invocations, num_functions))
inter_arrival_time = inter_arrival_time * 1000 # convert to ms
inter_arrival_time = np.ceil(inter_arrival_time)
#print(inter_arrival_time )

#inter_arrival_time_cumsum = np.cumsum(inter_arrival_time, axis=0)

fd = open(output_request_file, 'w')
for i in inter_arrival_time:
    json.dump({"interval": int(i), "function": "lorempy2", "payload":{"request": 42}}, fd)
    fd.write('\n')
fd.close()
