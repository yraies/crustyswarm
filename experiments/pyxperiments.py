#!/usr/bin/python
# vim: textwidth=120
from pathlib import Path
import copy
import json
import hashlib

resultdir="exps"

def debug(string):
    if False:
        print(string)

def rng(count,start,step):
    gens = [start + (i * step) for i in range(0,count)]
    return [round(i,6) for i in gens]

def rngobj(obj,count,start,step):
    return [{obj: round(start + (i * step),6)} for i in range(0,count)]

def change_in(old_base, str_path, value):
    def traverse_into(base, path, value):
        debug('Entering ' + str(path[0]))
        if len(path) > 1:
            return traverse_into(base[path[0]],path[1:],value)
        else:
            base[path[0]] = value
            return base
    base = copy.deepcopy(old_base)
    path = str_path.split('.')
    for v in range(0,len(path)):
        if path[v].isdigit():
            path[v] = int(path[v])
    traverse_into(base, path, value)
    return base

def norm(string):
    return string.replace('_','-')

def customHash(string):
    sha = hashlib.sha1()
    sha.update(bytes(string,'utf-8'))
    return sha.hexdigest()

def hasHashChanged(old_hashes, path, newhash):
    if path in old_hashes:
        return old_hashes[path] != newhash
    return True

Path(resultdir).mkdir(parents=True, exist_ok=True)
for p in Path(resultdir).glob('*.json'):
    p.unlink()

total_counter = 0
hashes={}

global_script = "#!/bin/bash\ncd exps\n"
all_script = "#!/bin/bash\ncd exps\n"
old_hashes = {}
try:
    old_hashes = json.loads(open(resultdir + "/hashes.hash", 'r').read())
except:
    print("no existing hashes found")

exps_json = open('pyxperiments.json', 'r').read()
configurations = json.loads(exps_json)

for base_config_name in configurations:
    print("Creating " + base_config_name + " configs")
    cargo_params = configurations[base_config_name]["cfg"]
    base_cfg = json.loads(open(base_config_name + '.json', 'r').read())
    # calculate hashes
    hashes[base_config_name]=customHash(json.dumps(base_cfg))
    base_hash_changed=False
    if hasHashChanged(old_hashes, base_config_name, hashes[base_config_name]):
        base_hash_changed=True
        print("Base Config " + base_config_name + " has Changed! " + hashes[base_config_name])
    #
    #   Single parameter Experiments
    #
    for experiment_name in configurations[base_config_name]["single"]:
        experiment = configurations[base_config_name]["single"][experiment_name]
        basename = norm(base_config_name) + '_' + norm(experiment[0])
        all_script +=  "./" + basename + ".sh \n"
        # look up hashes
        hashpath = base_config_name + "." + experiment_name
        newhash = customHash(json.dumps(experiment))
        hashes[hashpath] = newhash
        if not hasHashChanged(old_hashes, hashpath, newhash) and not base_hash_changed:
            print("Config for " + hashpath + " already exist")
            continue
        loc_script="#!/bin/bash\n"
        loc_script+= "rm -rf " + basename + "*/ \n"
        # Generate parameters
        params=[eval(x) for x in experiment[1:]]
        params=[z for y in params for z in y]
        try:
            # Notify when parameters are defined multiple times
            temparams=list(set(params))
            temparams.sort()
            if len(params) != len(temparams):
                print("\nSome generated parameters in " + experiment_name + " overlap!")
                print(params)
                print(temparams)
                print()
            params=temparams
        except:
            print("Could not uniquify/sort parameters in:")
        print(experiment_name.ljust(35,' ') + "\t" + str(len(params)) + "\t" + str(params) + "\t")
        for param_index in range(0,len(params)):
            filename = basename + '_' + str(param_index).rjust(2,'0') + '_' + norm(str(params[param_index])) + '.json'
            loc_script+="echo 'Running " + filename + "'\n sleep 1\ncargo run " + filename + " " + cargo_params + "\n"
            with open(resultdir + "/" + filename, 'w') as out_cfg:
                out_cfg.write(json.dumps(change_in(base_cfg, experiment_name, params[param_index])))
            total_counter += 1
            debug(loc_script)
        with open(resultdir + "/" + basename + ".sh", 'w') as out_sh:
            out_sh.write(loc_script)
        global_script += "./" + basename + ".sh \n"
    print()

global_script+="cd ..\n"
all_script+="cd ..\n"

with open(resultdir + "/hashes.hash", 'w') as out_hashes:
    out_hashes.write(json.dumps(hashes))
print("Generated " + str(total_counter) + " configurations")
with open(resultdir + "/runchanged.sh", 'w') as out_sh:
    out_sh.write(global_script)
with open(resultdir + "/runall.sh", 'w') as out_sh:
    out_sh.write(all_script)

for p in Path(resultdir).glob('*.sh'):
    p.chmod(0o754)

print(global_script)
