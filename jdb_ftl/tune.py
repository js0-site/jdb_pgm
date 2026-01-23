#!/usr/bin/env python3

import os
import subprocess
import ConfigSpace as CS
import ConfigSpace.hyperparameters as CSH
from dehb import DEHB

# Tuning Target
TRACE_FILE = "data/quick.bin"
BASE_THROUGHPUT = 250.0  # Baseline for quick.bin
BASE_P99 = 40.0         # Baseline for quick.bin

CARGO_CMD = [
    "cargo",
    "run",
    "--quiet",
    "--example",
    "score",
    "--release",
    "--",
    TRACE_FILE,
    str(BASE_THROUGHPUT),
    str(BASE_P99),
]

MIN_FIDELITY = 1
MAX_FIDELITY = 10
ITERATIONS = 50

REMAINING_COUNT = ITERATIONS
BEST_SCORE = 0.0

def target_function(config, fidelity=None, **kwargs):
    global REMAINING_COUNT, BEST_SCORE
    env = os.environ.copy()

    group_size = config["FTL_GROUP_SIZE"]
    buffer_cap = config["FTL_BUFFER_CAPACITY"]
    epsilon = config["FTL_PGM_EPSILON"]

    env["FTL_GROUP_SIZE"] = str(group_size)
    env["FTL_BUFFER_CAPACITY"] = str(buffer_cap)
    env["FTL_PGM_EPSILON"] = str(epsilon)

    try:
        # Run cargo with env vars
        result = subprocess.run(
            CARGO_CMD, env=env, check=True, capture_output=True, text=True
        )

        score_str = result.stdout.strip()
        lines = score_str.splitlines()
        score = 0.0
        if lines:
            try:
                # First line is score.
                # 第一行是分数。
                score = float(lines[0])
            except ValueError:
                return {"fitness": 1e12, "cost": 1.0}

        # Update best score.
        # 更新最高分。
        if score > BEST_SCORE:
            BEST_SCORE = score

        # Print progress: [Remaining] Parameters | Current Score | Best Score
        # 打印进度：[剩余次数] 参数 | 当前分数 | 最高分数
        print(
            f"[{REMAINING_COUNT:2d}] G={group_size:<4d} Buf={buffer_cap:<7d} Eps={epsilon:<3d} | "
            f"Score: {score:8.4f} | Best: {BEST_SCORE:8.4f}"
        )
        REMAINING_COUNT -= 1

        # Maximize Score => Minimize Fitness
        fitness = 1e9 / (score + 1e-6)
        return {"fitness": fitness, "cost": 1.0}

    except subprocess.CalledProcessError as e:
        REMAINING_COUNT -= 1
        return {"fitness": 1e12, "cost": 1.0}


def get_config_space():
    cs = CS.ConfigurationSpace()

    # FTL Group Size: 128 to 8192 (powers of 2)
    # Max seg_num = 32767, so GROUP_SIZE up to 16K is safe
    # FTL 分组大小：128 到 8192（2 的幂次）
    # 注意：GROUP_SIZE 的安全上限受限于 Segment 描述符中的 seg_num 字段（u16 / 2 = 15位，最大 32767）。
    # 每个 Group 的 Segment 数量不能超过 32767。如果出现最坏情况（每个 LBA 一个 Segment），即 GROUP_SIZE <= 32767。
    # 实际上，16K (16384) 是一个保守且安全的上限。
    group_size = CSH.OrdinalHyperparameter(
        "FTL_GROUP_SIZE", [128, 256, 512, 1024, 2048, 4096], default_value=2048
    )

    # FTL Buffer Capacity: 64K to 4M entries
    # 64K=65536 ... 4M=4194304
    # RocksDB default write_buffer_size is 64MB (~2M-4M entries)
    # FTL 缓冲区容量：64K 到 4M 条目
    buffer_cap = CSH.OrdinalHyperparameter(
        "FTL_BUFFER_CAPACITY", 
        [65536, 131072, 262144, 524288, 1048576, 2097152, 4194304], 
        default_value=1048576
    )

    # PGM Epsilon: 8 to 512
    # Larger epsilon = more tolerance = longer segments but less precision
    # PGM 误差容限：8 到 512
    epsilon = CSH.OrdinalHyperparameter(
        "FTL_PGM_EPSILON", [8, 16, 32, 64, 128, 256, 512], default_value=128
    )

    cs.add(group_size)
    cs.add(buffer_cap)
    cs.add(epsilon)
    return cs

def update_build_rs(config):
    # Since we use build.rs generated from env, update defaults in build.rs 
    # to match the found config is good practice, but technically not required 
    # if we just set env vars.
    # But the user asked: "tune.py会自动修改build.rs更新"
    
    # We will modify build.rs `get_env_or_default` fallback values.
    
    build_rs_path = "build.rs"
    if not os.path.exists(build_rs_path):
        return

    with open(build_rs_path, "r", encoding="utf-8") as f:
        content = f.read()

    import re
    
    for key, val in config.items():
        # search for get_env_or_default("KEY", 123usize)
        # We replace 123 with val
        
        # Regex: (get_env_or_default\s*\(\s*"{key}"\s*,\s*)(\d+)(usize\))
        pattern = f'(get_env_or_default\\s*\\(\\s*"{key}"\\s*,\\s*)(\\d+)(usize\\))'
        regex = re.compile(pattern)
        
        if regex.search(content):
            content = regex.sub(f"\\g<1>{val}\\g<3>", content)
            print(f"Updating {key} -> {val}")

    with open(build_rs_path, "w", encoding="utf-8") as f:
        f.write(content)
    print("Updated build.rs with best config.")

if __name__ == "__main__":
    cs = get_config_space()
    print(f"Starting Optimization...")
    
    # Check if we can run
    try:
        subprocess.run(["cargo", "--version"], stdout=subprocess.DEVNULL)
    except FileNotFoundError:
        print("Cargo not found.")
        exit(1)

    dehb = DEHB(
        f=target_function,
        cs=cs,
        dimensions=len(cs),
        min_fidelity=MIN_FIDELITY,
        max_fidelity=MAX_FIDELITY,
        n_workers=1,
        output_path="./dehb_logs",
        log_level="ERROR",
    )

    dehb.run(fevals=ITERATIONS)

    best_config = dehb.vector_to_configspace(dehb.inc_config)
    fitness = dehb.inc_score
    score = (1e9 / fitness) - 1e-6

    print("\n" + "=" * 40)
    print(f"Best Config Found:")
    print(f"Score: {score:.4f}")
    for k, v in best_config.items():
        print(f"  {k}: {v}")
    print("=" * 40 + "\n")

    update_build_rs(best_config)
