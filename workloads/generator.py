import numpy as np
from enum import Enum
import yaml
import operator
import functools
import random
import sys
import json

np.set_printoptions(threshold=sys.maxsize)

def finished(si, workload):
    for i,s in enumerate(si):
        if s < len(workload[i]) - 1:
            return False
    return True

## Generate request timestamps within a windown ([start,end]) from exponential
#  distribution with mean = mu
#
# Given a start timestamp and an end timestamp, generate the timestamps of requests
# whose inter-arrival time follows an exponential distribution/
# All timestamps will fall within [start, end] range. 
# 
# @param start Beginning of the window. All generated timestamps will be greater than
#              or equal to this value. This is not the timestamp of the first request
#              (although it's possible that it might be).
# @param end End of the window. All generated timestamps will be smaller than or equal
#            to this value.
# @param mu Mean of the exponential distribution. This represents the average inter-arrival
#           time in ms
def generate_request_timestamps(start, end, mu):
    duration = end - start
    expected_num_requests = int(duration/mu)

    inter_arrival_time = np.random.exponential(int(mu), (expected_num_requests, 1))
    inter_arrival_time = np.ceil(inter_arrival_time)
    inter_arrival_time_cumsum = np.cumsum(inter_arrival_time, axis=0)

    # shift all timestamps by `start` so that they fall within the window
    inter_arrival_time_cumsum = inter_arrival_time_cumsum + start
    inter_arrival_time_cumsum = inter_arrival_time_cumsum.astype(int)
    timestamp = inter_arrival_time_cumsum[inter_arrival_time_cumsum <= end]
    
    return timestamp




if __name__ == "__main__":
    # during non-spike periods, we assume each function has a steady stream
    # of requests coming in at 1 request per second or 0.001 request per ms
    default_arrival_rate = 0.001 
    default_mu = 1 / default_arrival_rate

    workload_config_file = sys.argv[1]
    output_request_file = sys.argv[2]
    print('loading workload config from: ' + workload_config_file)

    with open(workload_config_file) as f:
        config = f.read()

    data = yaml.load(config, Loader=yaml.Loader) # a list of dicts

    function_names = [f['name'] for f in data]
    mus = np.array([f['mu'] for f in data]) # average inter-arrival time in ms 
    start_times = np.array([f['start_time'] for f in data])
    end_times= np.array([f['end_time'] for f in data])
    arrival_rates = 1/mus # num of invocations per ms

    num_functions = len(arrival_rates)

    print('function names: ' + str(function_names))
    print('arrival rates: ' + str(arrival_rates) + "req/ms")
    print("mu:" +str(mus)+"ms")
    print('start time: ' + str(start_times)+"ms")
    print('end time: ' + str(end_times)+"ms")

    # Generate inter-arrival time for all functions
    max_end = end_times.max()
    # each element is a np.array() of timestamps for a particular function. Using
    # list instead of np.array() allows different sizes for each np.array
    workload = [] 

    for spike_start, spike_end, spike_mu in zip(start_times, end_times, mus):
        # during non-spike period, we assume that the function will have the
        # default_arrival_rate defined at the beginning. So for a function with
        # spike_start = 5000 and spike_end = 10000, we also need to generate
        # timestamps for [0, 5000] and (possibly) [10000, max_end]
        windows = [spike_start, spike_end]
        if spike_start > 0:
            windows.insert(0,0)

        if spike_end < max_end:
            windows.append(max_end)

        timestamp = []
        for i in range(len(windows)-1):
            mu = default_mu
            if windows[i] == spike_start:
                mu = spike_mu

            timestamp = np.append(timestamp, \
                                  generate_request_timestamps(windows[i], windows[i+1], mu))

        workload.append(timestamp)

    search_index = np.zeros(num_functions, dtype=np.int32)

    #inter_arrival_time_cumsum = inter_arrival_time_cumsum.astype(int)

    fd = open(output_request_file, 'w')

    pmin = 0

    while not finished(search_index, workload):
        candidates = [ workload[i][search_index[i]] for i in range(num_functions)]
        minv = np.min(candidates)
        min_idx = np.argmin(candidates)

        interval = minv - pmin
        pmin = minv

        json.dump({"interval": int(interval), "function": function_names[min_idx], "payload":{"request": 42}}, fd)
        fd.write('\n')

        search_index[min_idx] = search_index[min_idx] + 1

        if search_index[min_idx] == len(workload[min_idx]):
            search_index[min_idx] = search_index[min_idx] - 1
            workload[min_idx][search_index[min_idx]] = sys.maxsize


    fd.close()
