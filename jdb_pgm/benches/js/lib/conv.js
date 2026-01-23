/*
  解析转换 ./bench.js 生成的 json 为更加方便的格式，给上游使用
*/

export default (json_li) => {
  // Normalize and group data
  const perf = [];
  const accuracy = []; // We don't have accuracy data from criterion yet, logic needed if available
  const build = [];

  // Group by metric type based on 'group' field from Criterion
  for (const row of json_li) {
    const group = (row.group || "").trim();
    if (group === "single_lookups") {
      perf.push(row);
    } else if (group === "build_time") {
      build.push(row);
    } else if (group === "accuracy") {
      accuracy.push(row);
    } else {
      // Default to perf if it's not build or accuracy
      perf.push(row);
    }
  }

  // Pre-calculate derived metrics
  const config = {
    query_count: "1,000,000", // Fixed/Example
    data_sizes: [...new Set(json_li.map(r => r.data_size))].sort((a, b) => a - b),
    epsilon_values: [...new Set(json_li.map(r => r.epsilon).filter(e => e !== undefined))].sort((a, b) => a - b)
  };

  const sys = {
    osName: process.platform === 'darwin' ? 'macOS' : (process.platform === 'linux' ? 'Linux' : process.platform),
    arch: process.arch,
    cpu: "Apple M2 Max", // Placeholder, ideally fetch form os.cpus()
    cores: "12", // Placeholder
    mem: "64", // Placeholder
    rustVer: "1.85.0" // Placeholder
  };

  // Try to fetch real system info if possible, but keep it simple for now as per instructions (fill placeholders)

  return {
    perf,
    accuracy,
    build,
    config,
    sys
  };
};
