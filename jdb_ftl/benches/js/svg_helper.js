const COLORS = {
    p: "#3b82f6",
    s: "#10b981",
    w: "#f59e0b",
    d: "#ef4444",
    n: "#64748b",
    g: "#f1f5f9",
  },
  PALETTE = [COLORS.p, COLORS.s, COLORS.w, COLORS.d, "#8b5cf6", "#ec4899"],
  STYLES = `
  <style>
    .t { font: 600 18px Inter, system-ui, sans-serif; fill: #1e293b; }
    .l { font: 500 12px Inter, sans-serif; fill: #64748b; }
    .v { font: 600 12px Inter, sans-serif; fill: #334155; }
    .bl { font: 500 11px Inter, sans-serif; fill: #94a3b8; }
    .grid { stroke: ${COLORS.g}; stroke-width: 1; }
    .bar-bg { fill: ${COLORS.g}; }
    .shadow { filter: drop-shadow(0 2px 4px rgba(0,0,0,0.05)); }
  </style>
  <defs>
    <linearGradient id="gP" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="#3b82f6"/><stop offset="100%" stop-color="#60a5fa"/></linearGradient>
    <linearGradient id="gS" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="#10b981"/><stop offset="100%" stop-color="#34d399"/></linearGradient>
  </defs>
`,
  generateBarChart = (title, data, options = {}) => {
    const {
        width = 600,
        bar_h = 32,
        gap = 14,
        padding = 60,
        color = "url(#gP)",
      } = options,
      chart_h = data.length * (bar_h + gap) + padding * 2,
      max_v = Math.max(...data.map((d) => d.value), 1),
      label_w = 160,
      bar_w_max = width - padding * 2 - label_w - 40;

    let svg = `<svg width="${width}" height="${chart_h}" viewBox="0 0 ${width} ${chart_h}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;
    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${padding}" y="${padding - 20}" class="t">${title}</text>`;

    data.forEach((d, i) => {
      const y = padding + i * (bar_h + gap),
        bw = (d.value / max_v) * bar_w_max;
      svg += `<text x="${padding}" y="${y + bar_h / 2 + 5}" class="l">${d.label}</text>`;
      svg += `<rect x="${padding + label_w}" y="${y}" width="${bar_w_max}" height="${bar_h}" class="bar-bg" />`;
      svg += `<rect x="${padding + label_w}" y="${y}" width="${bw}" height="${bar_h}" fill="${color}" class="shadow" />`;
      svg += `<text x="${padding + label_w + bw + 12}" y="${y + bar_h / 2 + 5}" class="v">${d.value.toLocaleString()}</text>`;
      if (d.suffix)
        svg += `<text x="${padding + label_w + bw + 110}" y="${y + bar_h / 2 + 5}" class="bl" text-anchor="end">${d.suffix}</text>`;
    });
    return svg + `</svg>`;
  },
  generateDonutChart = (title, data, options = {}) => {
    const {
        width = 450,
        height = 350,
        r = 90,
        ir = 65,
        padding = 50,
      } = options,
      cx = width / 2 - 40,
      cy = height / 2 + 20,
      total = data.reduce((s, d) => s + d.value, 0);
    let start_a = -Math.PI / 2,
      svg = `<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;

    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${width / 2}" y="${padding}" class="t" text-anchor="middle">${title}</text>`;

    data.forEach((d, i) => {
      const pct = d.value / total,
        end_a = start_a + pct * 2 * Math.PI,
        x1 = cx + r * Math.cos(start_a),
        y1 = cy + r * Math.sin(start_a),
        x2 = cx + r * Math.cos(end_a),
        y2 = cy + r * Math.sin(end_a),
        ix1 = cx + ir * Math.cos(start_a),
        iy1 = cy + ir * Math.sin(start_a),
        ix2 = cx + ir * Math.cos(end_a),
        iy2 = cy + ir * Math.sin(end_a),
        large = pct > 0.5 ? 1 : 0,
        p = `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2} L ${ix2} ${iy2} A ${ir} ${ir} 0 ${large} 0 ${ix1} ${iy1} Z`;

      svg += `<path d="${p}" fill="${PALETTE[i % PALETTE.length]}" class="shadow" />`;
      const ly = padding + 60 + i * 26,
        lx = width - 140;
      svg += `<rect x="${lx}" y="${ly}" width="12" height="12" fill="${PALETTE[i % PALETTE.length]}" />`;
      svg += `<text x="${lx + 20}" y="${ly + 10}" class="l">${d.label}</text>`;
      svg += `<text x="${lx + 20}" y="${ly + 22}" class="bl">${(pct * 100).toFixed(1)}%</text>`;
      start_a = end_a;
    });

    svg += `<text x="${cx}" y="${cy - 5}" class="bl" text-anchor="middle">TOTAL</text>`;
    svg += `<text x="${cx}" y="${cy + 15}" class="v" text-anchor="middle" font-size="20">${total.toLocaleString()}</text>`;
    return svg + `</svg>`;
  },
  generateGauge = (title, val, options = {}) => {
    const {
        width = 300,
        height = 220,
        min = 0,
        max = 100,
        suffix = "%",
      } = options,
      cx = width / 2,
      cy = height - 50,
      r = 85,
      ir = 70,
      pct = Math.min(Math.max((val - min) / (max - min), 0), 1),
      a = Math.PI + pct * Math.PI,
      tx = cx + r * Math.cos(a),
      ty = cy + r * Math.sin(a),
      itx = cx + ir * Math.cos(a),
      ity = cy + ir * Math.sin(a);

    let svg = `<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;
    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${cx}" y="40" class="t" text-anchor="middle">${title}</text>`;

    svg += `<path d="M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy} L ${cx + ir} ${cy} A ${ir} ${ir} 0 0 0 ${cx - ir} ${cy} Z" fill="${COLORS.g}" />`;
    const pColor = pct > 0.8 ? COLORS.s : pct > 0.5 ? COLORS.w : COLORS.d;
    svg += `<path d="M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${tx} ${ty} L ${itx} ${ity} A ${ir} ${ir} 0 0 0 ${cx - ir} ${cy} Z" fill="${pColor}" class="shadow" />`;

    svg += `<text x="${cx}" y="${cy - 15}" class="v" text-anchor="middle" font-size="28">${val}${suffix}</text>`;
    svg += `<text x="${cx - r}" y="${cy + 20}" class="bl" text-anchor="middle">${min}</text>`;
    svg += `<text x="${cx + r}" y="${cy + 20}" class="bl" text-anchor="middle">${max}</text>`;
    return svg + `</svg>`;
  },
  generateLineChart = (title, data, options = {}) => {
    const {
        width = 600,
        height = 380,
        padding = 70,
        xLabel = "",
        yLabel = "",
        logX = false,
      } = options,
      cw = width - padding * 2,
      ch = height - padding * 2,
      minX = Math.min(...data.map((d) => d.x)),
      mx = Math.max(...data.map((d) => d.x), 1),
      my = Math.max(...data.map((d) => d.y), 1);

    // Helper for logarithmic X positioning
    const xPos = logX
      ? (x) => padding + (Math.log10(x) / Math.log10(mx)) * cw
      : (x) => padding + (x / mx) * cw;

    let svg = `<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;
    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${padding}" y="${padding - 30}" class="t">${title}</text>`;

    // Y-axis grid and labels
    for (let i = 0; i <= 4; i++) {
      const gy = height - padding - (i / 4) * ch;
      svg += `<line x1="${padding}" y1="${gy}" x2="${width - padding}" y2="${gy}" class="grid" />`;
      svg += `<text x="${padding - 10}" y="${gy + 4}" class="bl" text-anchor="end">${Math.round((i / 4) * my)}</text>`;
    }

    // X-axis tick labels
    data.forEach((d) => {
      const px = xPos(d.x);
      svg += `<text x="${px}" y="${height - padding + 20}" class="bl" text-anchor="middle">${d.x}</text>`;
    });

    // X-axis label
    if (xLabel) {
      svg += `<text x="${width / 2}" y="${height - 15}" class="l" text-anchor="middle">${xLabel}</text>`;
    }

    // Y-axis label (rotated)
    if (yLabel) {
      svg += `<text x="15" y="${height / 2}" class="l" text-anchor="middle" transform="rotate(-90, 15, ${height / 2})">${yLabel}</text>`;
    }

    let p = "",
      area = `M ${padding} ${height - padding}`;
    data.forEach((d, i) => {
      const px = xPos(d.x),
        py = height - padding - (d.y / my) * ch;
      p += (i === 0 ? "M " : " L ") + `${px} ${py}`;
      area += ` L ${px} ${py}`;
      if (i === data.length - 1) area += ` L ${px} ${height - padding} Z`;
      svg += `<circle cx="${px}" cy="${py}" r="4" fill="${COLORS.p}" class="shadow" />`;
    });

    svg += `<path d="${area}" fill="${COLORS.p}" fill-opacity="0.1" />`;
    svg += `<path d="${p}" fill="none" stroke="${COLORS.p}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="shadow" />`;
    return svg + `</svg>`;
  },
  generateStackedBarChart = (title, data, options = {}) => {
    const { width = 600, bar_h = 36, padding = 70 } = options,
      label_w = 160,
      bar_w_max = width - padding * 2 - label_w;
    let svg = `<svg width="${width}" height="${data.length * (bar_h + 24) + padding * 2}" viewBox="0 0 ${width} ${data.length * (bar_h + 24) + padding * 2}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;

    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${padding}" y="${padding - 25}" class="t">${title}</text>`;

    data.forEach((row, ri) => {
      const y = padding + ri * (bar_h + 24),
        total = row.stacks.reduce((s, st) => s + st.value, 0);
      let cur_x = padding + label_w;
      svg += `<text x="${padding}" y="${y + bar_h / 2 + 5}" class="l">${row.label}</text>`;
      svg += `<rect x="${cur_x}" y="${y}" width="${bar_w_max}" height="${bar_h}" class="bar-bg" />`;
      row.stacks.forEach((st, si) => {
        const sw = (st.value / total) * bar_w_max;
        svg += `<rect x="${cur_x}" y="${y}" width="${sw}" height="${bar_h}" fill="${PALETTE[si % PALETTE.length]}" />`;
        cur_x += sw;
      });
    });

    if (data[0]?.stacks) {
      data[0].stacks.forEach((st, i) => {
        const lx = padding + i * 130,
          ly = data.length * (bar_h + 24) + padding + 15;
        svg += `<rect x="${lx}" y="${ly}" width="12" height="12" fill="${PALETTE[i % PALETTE.length]}" />`;
        svg += `<text x="${lx + 20}" y="${ly + 10}" class="bl">${st.label}</text>`;
      });
    }
    return svg + `</svg>`;
  },
  generateHeatmap = (title, data, options = {}) => {
    const { width = 600, height = 200, padding = 40, rows = 10 } = options,
      cols = Math.ceil(data.length / rows),
      cell_size = (width - padding * 2) / cols,
      max_v = Math.max(...data, 1);

    let svg = `<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">${STYLES}`;
    svg += `<rect width="100%" height="100%" fill="white" />`;
    svg += `<text x="${padding}" y="${padding - 10}" class="t">${title}</text>`;

    data.forEach((v, i) => {
      const r = i % rows,
        c = Math.floor(i / rows),
        x = padding + c * cell_size,
        y = padding + 15 + r * cell_size,
        opacity = 0.1 + (v / max_v) * 0.9;
      svg += `<rect x="${x}" y="${y}" width="${cell_size - 1}" height="${cell_size - 1}" fill="${COLORS.p}" fill-opacity="${opacity}" />`;
    });
    return svg + `</svg>`;
  };

export default {
  generateBarChart,
  generateDonutChart,
  generateStackedBarChart,
  generateGauge,
  generateLineChart,
  generateHeatmap,
};
