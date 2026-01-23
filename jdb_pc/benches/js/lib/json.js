
import { readFile } from "fs/promises";

export const parseBenchJson = async (path) => {
    const content = await readFile(path, "utf-8");
    const lines = content.trim().split("\n");
    const results = [];

    for (const line of lines) {
        try {
            const json = JSON.parse(line);
            if (json.reason === "benchmark-complete") {
                results.push(json);
            } else if (json.reason === "custom-metric") {
                results.push({
                    id: json.id,
                    reason: "benchmark-complete",
                    mean: { estimate: json.estimate }
                });
            }
        } catch (e) {
            // ignore empty or invalid lines
        }
    }
    return results;
};

export const formatTime = (ns) => {
    if (ns < 1000) return `${ns.toFixed(2)} ns`;
    if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)} µs`;
    if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)} ms`;
    return `${(ns / 1_000_000_000).toFixed(2)} s`;
};
