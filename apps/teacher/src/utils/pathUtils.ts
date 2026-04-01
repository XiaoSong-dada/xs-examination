/**
 * 路径去重与规范化工具
 * - 去空、去重、去掉 file:// 前缀
 * - 将反斜杠统一为正斜杠，合并重复斜杠
 * - 去掉开头的斜杠以返回相对路径
 */
export function dedupePaths(paths: string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of paths) {
    let p = (raw ?? "").trim();
    if (!p) continue;

    // 去掉 file:// 或 file:/// 前缀
    p = p.replace(/^file:\/\/+/, "");

    // Windows 路径统一为 /
    p = p.replace(/\\/g, "/");

    // 合并连续的 /
    p = p.replace(/\/+/g, "/");

    // 去掉开头的 /，保持相对路径形式（按需可改）
    if (p.startsWith("/")) p = p.slice(1);

    if (!seen.has(p)) {
      seen.add(p);
      out.push(p);
    }
  }
  return out;
}

export default dedupePaths;
