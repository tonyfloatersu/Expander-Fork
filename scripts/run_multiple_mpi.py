import sys
import json
import subprocess

MPI_CONFIG_JSON = '''
{
    "field": "m31ext3",
    "n_groups": 1,
    "mpi_size_each_group": 8
}
'''

def parse_config(mpi_config):
    field = mpi_config["field"]
    n_groups = mpi_config["n_groups"]
    mpi_size_each_group = mpi_config["mpi_size_each_group"]
    
    if field not in ["gf2ext128", "m31ext3", "fr"]:
        sys.exit("Unrecognized field, now only supports gf2ext128, m31ext3 and fr")

    if n_groups <= 0:
        sys.exit("Number of groups should be positive")
    cpu_ids = []
    for i in range(n_groups):
        cpu_ids.append(list(range(i * mpi_size_each_group, (i + 1) * mpi_size_each_group)))
    for i in range(n_groups):
        if len(cpu_ids[i]) != mpi_size_each_group:
            sys.exit(f"Cpu ids are not correct for group {i}")

    return field, n_groups, mpi_size_each_group, cpu_ids


# Run two mpi process
if __name__ == "__main__":
    mpi_config = json.loads(MPI_CONFIG_JSON)
    field, n_groups, mpi_size_each_group, cpu_ids = parse_config(mpi_config)

    for i in range(n_groups):
        cpu_id = ",".join(map(str, cpu_ids[i]))
        subprocess.Popen(["mpiexec", "-cpu-set", cpu_id, "-n", str(mpi_size_each_group), "./target/release/gkr-mpi", "-f", field, "-c",  "../../EthFullConsensus/consensus/shuffle/gkr/circuit_shufflewithhashmap128tobinary.txt", "-w",  "../../EthFullConsensus/consensus/shuffle/gkr/witness_shufflewithhashmap128tobinary.txt"])
        # subprocess.Popen(["mpiexec", "-cpu-set", cpu_id, "-n", str(mpi_size_each_group), "./target/release/expander-exec", "-- serve", "../../EthFullConsensus/consensus/shuffle/gkr/circuit_shufflewithhashmap128tobinary.txt",  "127.0.0.1 3030"])
