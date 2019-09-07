import json
import yaml
import sys
import numpy as np
import matplotlib.pyplot as plt
np.set_printoptions(threshold=sys.maxsize)

SMALLEST_VM = 128
NS2MS = 1000000

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
        if self.evict == []:
            return 0

        return self.evict[0][1] - self.evict[0][0]
    
    def idle_time(self):
        """Return the total amount of time that this VM is up but not running app code"""
        return self.uptime() - self.runtime()

    def uptime(self):
        """Return the total amount of time that VM is up"""
        if self.evict == []:
            return end_time - self.boot[0][1]

        return self.evict[0][0] - self.boot[0][1]

    def uptimestamp(self):
        """Return the launch finish timestamp and shutdown start timestamp of this VM in a tuple"""
        if self.evict == []:
            return (self.boot[0][1], end_time)

        return (self.boot[0][1], self.evict[0][0])

    def lifetime(self):
        """Return the total amount of time between start VM command and eviction finishes"""
        if self.evict == []:
            return end_time - self.boot[0][0]

        return self.evict[0][1] - self.boot[0][0]

    def lifetimestamp(self):
        """Return the launch start timestamp and shutdown finish timestamp of this VM in a tuple"""
        if self.evict == []:
            return (self.boot[0][0], end_time)

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

start_time = data['start time']/NS2MS
end_time = data['end time']/NS2MS
num_vm = len(data['boot timestamps'])
total_mem = data['total mem']
resource_limit = int(total_mem/SMALLEST_VM) # the maximum number of 128MB VMs that the cluster can support

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

# scheduler latency
schedule_latency = np.array(data['request schedule latency'])
schedule_latency = schedule_latency / NS2MS

# calculate high-level aggregate data
vms = []
all_req_res = []
all_eviction_tsp = []
all_boot_tsp = []
for vm_id in range(3, 3+num_vm,1):
    mem_size = data['vm mem sizes'][str(vm_id)]
    boot_tsp = [l/NS2MS for l in data['boot timestamps'][str(vm_id)]]
    req_res_tsp = [l/NS2MS for l in data['request/response timestamps'][str(vm_id)]]
    all_req_res = all_req_res+req_res_tsp
    all_boot_tsp = all_boot_tsp + boot_tsp

    try:
        evict_tsp = [l/NS2MS for l in data['eviction timestamps'][str(vm_id)]]
        all_eviction_tsp = all_eviction_tsp + evict_tsp
    except:
        evict_tsp = []

    vm = VM(vm_id, boot_tsp, req_res_tsp, evict_tsp, mem_size)
    vms.append(vm)


total_idle_time = 0
total_boot_time = 0
total_eviction_time = 0
total_runtime = 0
total_runtimeMB = 0
total_idle_timeMB = 0
total_eviction_timeMB = 0
total_boot_timeMB = 0

for vm in vms:
    total_idle_time += vm.idle_time()
    total_idle_timeMB += vm.idle_time() * vm.resource
    total_boot_time += vm.boot_time()
    total_boot_timeMB += vm.boot_time() * vm.resource
    total_eviction_time += vm.evict_time()
    total_eviction_timeMB += vm.evict_time() * vm.resource
    total_runtime += vm.runtime()
    total_runtimeMB += vm.runtime() * vm.resource

#    print("vm {}, uptime: {}, runtime: {}, idle time: {}, boot time: {}, evict time: {}"\
#            .format(vm.id,\
#                vm.uptime()/1000000,\
#                vm.runtime()/1000000,
#                vm.idle_time()/1000000,\
#                vm.boot_time()/1000000,\
#                vm.evict_time()/1000000))

total_time = total_runtime + total_idle_time + total_boot_time + total_eviction_time
total_experiment_duration = (end_time - start_time)

print("cluster size: {}MB".format(total_mem))
print('cluster can support ' + str(resource_limit) + ' 128MB VMs')
print("booted a total of " + str(num_vm) + " VMs")
print('number of completed requests: {}'.format(data['number of completed requests']))
print('number of dropped requests (resource exhaustion): {}'.format(data['drop requests (resource)']))
print('number of dropped requests (concurrency limit): {}'.format(data['drop requests (concurrency)']))
print('number of evictions: {}'.format(data['number of evictions']))
print('cumulative throughput: {}'.format(data['cumulative throughput']))
print("experiment duration: {}ms".format(int(total_experiment_duration)))
print("total time (spent by all VMs): {}ms".format(int(total_time)))
print("total runtime time: {}ms".format(int(total_runtime)))
print("total idle time: {}ms".format(int(total_idle_time)))
print("total boot time: {}ms".format(int(total_boot_time)))
print("total eviction time: {}ms".format(int(total_eviction_time)))
#print("total runtimeMB (ms-MB): {}ms-MB".format(int(total_runtimeMB)))
print("type 1 utilization: {}".format(total_runtimeMB/(total_experiment_duration*total_mem)))
print("type 2 utilization: {}".format(total_runtimeMB/(total_runtimeMB + total_eviction_timeMB + total_boot_timeMB)))

print("average scheduling latency: {}ms".format(np.mean(schedule_latency)))


# calculate utilization over the timespan of the experiment
count_running = []
runtimemb_all = []
window_size = 5000 #ms

wt = start_time + window_size / 2
while wt < end_time:
    window = (wt - window_size / 2, wt + window_size / 2)
    running = 0
    runtimemb = 0
    for vm in vms:
        runtimemb = runtimemb + vm.runtime_in_window(window) * vm.resource

    runtimemb_all.append(runtimemb/(window_size*total_mem))

    wt = wt + window_size

#utilization = np.array(count_running)
#utilization = utilization/resource_limit*100
utilization = np.array(runtimemb_all) * 100

# plot
x = np.linspace(0, (end_time - start_time ), len(utilization)  )

fig = plt.figure()
fig.set_size_inches(8,5)
plt.plot(x, utilization)
plt.xlabel('time(ms)')
plt.ylabel('Utilization (%)')
plt.title('Utilization')
plt.legend()
plt.savefig('test.png')

plt.show()

