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
    return tsp>=tuple_range[0] and tsp<=tuple_range[1]

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
window_size = 3000000000 #ns
ws = start_time + window_size / 2 # sampling tick
i = 0
while ws < end_time:
    running = 0
    for vm in vms:
        running = running + vm.is_running(ws)

    count_running.append(running)

    ws = ws + window_size

utilization = np.array(count_running)
utilization = utilization/resource_limit*100

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
