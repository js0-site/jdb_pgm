
export const ALGORITHM_COLORS = {
    jdb_pgm: "#ff5722", // OrangeRed
    pgm_index: "#91cc75",
    external_pgm: "#91cc75",
    binary_search: "#fac858",
    btreemap: "#8d6e63", // Brown
    hashmap: "#73c0de",
    default: "#5470c6"
};

export const getColor = (name) => {
    // Handle variants like "jdb_pgm (e=32)" -> "jdb_pgm"
    const key = Object.keys(ALGORITHM_COLORS).find(k => name.includes(k));
    return ALGORITHM_COLORS[key] || ALGORITHM_COLORS.default;
};
