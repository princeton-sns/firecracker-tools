import json
import yaml
import sys
import numpy as np
import matplotlib.pyplot as plt

SMALLEST_VM = 128

def list_to_tuple_list(l):
    if len(l) == 0:
        return l

    if len(l) % 2 == 1:
        sys.exit("list has odd number of elements")

    return [(l[i], l[i+1]) for i in range(0,len(l),2)]

def in_tuple_range(tsp, tuple_range):
    """Return whether a timestamp falls within a time range

    tsp -- timestamp
    tuple_range -- a tuple of (start timestamp, end timestamp)
    """
    return tsp>=tuple_range[0] and tsp<=tuple_range[1]

def overlap(window, time_range):
    """Return the amount of overlap time between 2 time ranges

    0 if all of time_range precedes window
    -1 if all of time_range follows window
    overlap amount otherwise
    """
    if time_range[1] <= window[0]:
        return 0

    if time_range[0] >= window[1]:
        return -1

    l = [time_range[0], time_range[1], window[0], window[1]]
    l.sort()

    return l[2] - l[1]

class VM(object):
    def __init__(self, id, boot_tsp, req_res_tsp, evict_tsp, resource):
        self.id = id
        self.boot = list_to_tuple_list(boot_tsp)
        self.req_res = list_to_tuple_list(req_res_tsp)
        self.evict = list_to_tuple_list(evict_tsp)
        self.stage = 0 
        self.req_res_idx = 0
        self.resource = resource

    def __str__(self):
        return "id: " + str(self.id) + " boot: " + str(self.boot) + \
                " req_res: " + str(self.req_res) + " evict: " + str(self.evict) +\
                " resource size: " + str(self.resource)

    def is_running(self, tsp):

        for req_res_tuple in self.req_res:
            if in_tuple_range(tsp, req_res_tuple):
                return self.resource

        return 0

    def boot_time(self):
        """Return the total amount of time that this VM spent in booting"""
        return self.boot[0][1] - self.boot[0][0]

    def runtime(self):
        """Return the total amount of time that this VM spent running app code"""
        runtime = 0
        for r in self.req_res:
            runtime = runtime + (r[1] - r[0])

        return runtime

    def evict_time(self):
        """Return the total amount of time that this VM spent in eviction"""
        return self.evict[0][1] - self.evict[0][0]
    
    def idle_time(self):
        """Return the total amount of time that this VM is up but not running app code"""
        return self.uptime() - self.runtime()

    def uptime(self):
        """Return the total amount of time that VM is up"""
        if self.evict == []:
            return end_time - self.boot[0][1]
        else:
            return self.evict[0][0] - self.boot[0][1]

    def uptimestamp(self):
        """Return the launch finish timestamp and shutdown start timestamp of this VM in a tuple"""
        return (self.boot[0][1], self.evict[0][0])

    def lifetime(self):
        """Return the total amount of time between start VM command and eviction finishes"""
        return self.evict[0][1] - self.boot[0][0]

    def lifetimestamp(self):
        """Return the launch start timestamp and shutdown finish timestamp of this VM in a tuple"""
        return (self.boot[0][0], self.evict[0][1])

    def runtime_in_window(self, window):
        """Return the amount of time within a window that this VM spent running app code

        windown -- a tuple representing a window
        """
        runtime = 0
        for r in self.req_res:
            ol = overlap(window, r)
            if ol == -1:
                break
            runtime = runtime + ol

        return runtime



measurement_file = open(sys.argv[1], 'r')
data = json.load(measurement_file)

measurement_file.close()

start_time = data['start time']
end_time = data['end time']
num_vm = len(data['boot timestamps'])

#function_config_file = open(sys.argv[2], 'r')
#config = yaml.load(function_config_file.read(), Loader=yaml.Loader)
#function_config_file.close()
# get mem size for each function
#function_to_memsize = {}
#for function in config:
#    name = function['name']
#    mem = function['memory']
#    function_to_memsize[name] = mem
#
#print(function_to_memsize)


# calculate high-level aggregate data
vms = []
all_req_res = []
all_eviction_tsp = []
all_boot_tsp = []
for vm_id in range(3, 3+num_vm,1):
    mem_size = data['vm mem sizes'][str(vm_id)]
    boot_tsp = data['boot timestamps'][str(vm_id)]
    req_res_tsp = data['request/response timestamps'][str(vm_id)]
    all_req_res = all_req_res+req_res_tsp
    all_boot_tsp = all_boot_tsp + boot_tsp

    try:
        evict_tsp = data['eviction timestamps'][str(vm_id)]
        all_eviction_tsp = all_eviction_tsp + evict_tsp
    except:
        evict_tsp = []

    vm = VM(vm_id, boot_tsp, req_res_tsp, evict_tsp, mem_size/SMALLEST_VM)
    vms.append(vm)

all_runtime = [all_req_res[i+1] - all_req_res[i] for i in range(0, len(all_req_res)-1, 2)]
all_runtime = np.array(all_runtime)/1000000

all_boot = [all_boot_tsp[i+1] - all_boot_tsp[i] for i in range(0, len(all_boot_tsp)-1, 2)]
all_boot= np.array(all_boot)/1000000

all_eviction= [all_eviction_tsp[i+1] - all_eviction_tsp[i] for i in range(0, len(all_eviction_tsp)-1, 2)]
all_eviction= np.array(all_eviction)/1000000


total_runtime = np.sum(all_runtime)
total_experiment_time = (end_time - start_time) / 1000000
total_eviction_time = np.sum(all_eviction)
total_boot_time = np.sum(all_boot)
total_mem = data['total mem']
resource_limit = int(total_mem/SMALLEST_VM) # the maximum number of 128MB VMs that the cluster can support
num_drop_res = data['drop requests (resource)']
num_drop_concur = data['drop requests (concurrency)']

print("booted a total of " + str(num_vm) + " VMs")
print("total experiment time: {}".format(total_experiment_time))
print("total runtime: {}".format(total_runtime))
print("total eviction time: {}".format(total_eviction_time))
print("total boot time: {}".format(total_boot_time))
print("type 1 utilization: {}".format(total_runtime * 256/(total_experiment_time*1024)))
print("type 2 utilization: {}".format(total_runtime/(total_runtime + total_eviction_time + total_boot_time)))

print('number of completed requests: {}'.format(data['number of completed requests']))
print('number of dropped requests (resource exhaustion): {}'.format(num_drop_res))
print('number of dropped requests (concurrency limit): {}'.format(num_drop_concur))
print('number of evictions: {}'.format(data['number of evictions']))
print('cumulative throughput: {}'.format(data['cumulative throughput']))

print('cluster can support ' + str(resource_limit) + ' 128MB VMs')


# calculate utilization over the timespan of the experiment
count_running = []
runtimemb_all = []
window_size = 500 #ms
window_size = window_size * 1000000

wt = start_time + window_size / 2
while wt < end_time:
    window = (wt - window_size / 2, wt + window_size / 2)
    running = 0
    runtimemb = 0
    for vm in vms:
        running = running + vm.is_running(wt)
        runtimemb = runtimemb + vm.runtime_in_window(window) * vm.resource

    count_running.append(running)
    runtimemb_all.append(runtimemb/(window_size*resource_limit))

    wt = wt + window_size

#utilization = np.array(count_running)
#utilization = utilization/resource_limit*100
utilization = np.array(runtimemb_all) * 100

# plot
x = np.linspace(0, (end_time - start_time )/1000000, len(utilization)  )

fig = plt.figure()
fig.set_size_inches(8,5)
plt.plot(x, utilization)
plt.xlabel('time(ms)')
plt.ylabel('Utilization (%)')
plt.title('Utilization')
plt.legend()
plt.savefig('test.png')

plt.show()
