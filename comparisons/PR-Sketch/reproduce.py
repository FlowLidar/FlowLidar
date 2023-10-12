#!/usr/bin/python3
import numpy as np
import os
import argparse

zero_error_array = []
aae_array = []
are_array = []
sent_keys_array = []

parser = argparse.ArgumentParser()
parser.add_argument("--script", type=str, default="pr_sketch_test")
parser.add_argument("--filter_memory", type=int, default=64)
parser.add_argument("--sketch_memory", type=int, default=64)
parser.add_argument("--filter_hash_nr", type=int, default=1)
parser.add_argument("--sketch_hash_nr", type=int, default=1)
args = parser.parse_args()
directory = "./comparisons/PR-Sketch"

for i in range(60):
    #print("./{} 64 64 ./data/CAIDA_trace_{}/1000ms/split_{}.bin 2>&1".format(args.script,j,i))
    # check file is not empty
    if os.stat("{}/data/CAIDA_trace_1/1000ms/split_{}.bin".format(directory,i)).st_size == 0:
        continue
    #print("{}/{} {} {} {}/data/CAIDA_trace_1/1000ms/split_{}.bin {} {} 2>&1".format(directory,args.script,args.filter_memory,args.sketch_memory,directory,i,args.filter_hash_nr, args.sketch_hash_nr))
    stream =  os.popen("{}/{} {} {} {}/data/CAIDA_trace_1/1000ms/split_{}.bin {} {} 2>&1".format(directory,args.script,args.filter_memory,args.sketch_memory,directory,i,args.filter_hash_nr, args.sketch_hash_nr))
    lines = stream.readlines()
    zero_error = float((lines[9].split(" "))[4].strip())
    aae = float((lines[13].split(" "))[1].strip())
    are = float((lines[14].split(" "))[1].strip())
    sent_keys = int((lines[18].split(" "))[5].strip())
    zero_error_array.append(zero_error)
    aae_array.append(aae)
    are_array.append(are)
    sent_keys_array.append(sent_keys)
    #print(zero_error, aae, are, sent_keys)
    #print(i)
print("{}\t{}\t{}\t{}".format(100*np.array(zero_error_array).mean(), np.array(aae_array).mean(), np.array(are_array).mean(), np.array(sent_keys_array).mean()))
zero_error_array.clear()
aae_array.clear()
are_array.clear()
sent_keys_array.clear()


