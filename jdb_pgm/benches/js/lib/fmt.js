
export const formatDataSize = (cnt) => {
    cnt = Number(cnt);
    if (cnt >= 1e6) {
        return (cnt / 1e6).toFixed(0) + "M";
    }
    if (cnt >= 1e3) {
        return (cnt / 1e3).toFixed(0) + "K";
    }
    return cnt.toString();
};

export const fmtTime = (ns) => {
    if (ns < 1000) return `${ns.toFixed(2)}ns`;
    if (ns < 1e6) return `${(ns / 1000).toFixed(2)}µs`;
    if (ns < 1e9) return `${(ns / 1e6).toFixed(2)}ms`;
    return `${(ns / 1e9).toFixed(2)}s`;
};

export const fmtThroughput = (val) => {
    // User requested "Million" unit.
    if (val > 1e6) return `${(val / 1e6).toFixed(2)} M/s`;
    if (val > 1e3) return `${(val / 1e3).toFixed(2)} K/s`;
    return `${val.toFixed(2)} /s`;
};

export const formatMemory = (bytes) => {
    if (!bytes) return "0";
    const mb = bytes / (1024 * 1024);
    if (mb < 0.01) return "< 0.01";
    return mb.toFixed(2);
};
