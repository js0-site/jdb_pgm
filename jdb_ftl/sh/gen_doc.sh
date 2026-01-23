#!/bin/bash
# 生成文档：合并 readme/zh/ 下的所有 Markdown 文件并转换为 HTML 和 PDF

set -e

ROOT_DIR="/Users/z/js0/jdb_pgm/jdb_ftl"
GEN_DIR="${ROOT_DIR}/gen"
TEMP_MD="${ROOT_DIR}/.temp_doc.md"

echo "开始生成文档..."

# 合并所有 Markdown 文件
echo "合并文档..."
cat "${ROOT_DIR}/readme/zh.md" > "${TEMP_MD}"
echo "" >> "${TEMP_MD}"
echo "---" >> "${TEMP_MD}"
echo "" >> "${TEMP_MD}"

# 按顺序合并核心文档
for file in "bench.md" "architecture.md" "codec.md" "structures.md" "flush.md" "benchmark.md" "tuning.md"; do
    if [ -f "${ROOT_DIR}/readme/zh/${file}" ]; then
        echo "添加: ${file}"
        cat "${ROOT_DIR}/readme/zh/${file}" >> "${TEMP_MD}"
        echo "" >> "${TEMP_MD}"
        echo "---" >> "${TEMP_MD}"
        echo "" >> "${TEMP_MD}"
    fi
done

# 生成 HTML
echo "生成 HTML..."
pandoc "${TEMP_MD}" \
    -o "${GEN_DIR}/doc.html" \
    --standalone \
    --katex \
    --toc \
    --toc-depth=3 \
    --metadata title="JDB-FTL 文档"

echo "✓ HTML 生成完成: ${GEN_DIR}/doc.html"

# 尝试生成 PDF（如果 xelatex 可用）
if command -v xelatex &> /dev/null; then
    echo "生成 PDF..."
    pandoc "${TEMP_MD}" \
        -o "${GEN_DIR}/doc.pdf" \
        --pdf-engine=xelatex \
        --toc \
        --toc-depth=3 \
        --metadata title="JDB-FTL 文档" \
        -V CJKmainfont="PingFang SC" \
        -V geometry:margin=1in
    echo "✓ PDF 生成完成: ${GEN_DIR}/doc.pdf"
else
    echo "⚠ xelatex 未安装，跳过 PDF 生成"
    echo "  如需生成 PDF，请安装 LaTeX: brew install --cask mactex"
fi

# 清理临时文件
rm -f "${TEMP_MD}"

echo "文档生成完成！"