import { useCallback } from "react";

import { resolveImageAssetPreview } from "@/services/fileAssetService";

/**
 * 提供学生端图片预览地址解析能力。
 *
 * @returns 返回批量解析图片预览地址的方法。
 */
export function useFileHooks() {
  /**
   * 批量解析图片相对路径为可渲染的预览地址。
   *
   * @param relativePaths 题目与选项中的图片路径列表。
   * @returns 返回以相对路径为 key 的预览地址映射。
   */
  const resolveImagePreviews = useCallback(async (
    relativePaths: string[],
  ): Promise<Record<string, string>> => {
    if (relativePaths.length === 0) {
      return {};
    }

    const uniquePaths = Array.from(new Set(relativePaths.map((item) => item.trim()).filter(Boolean)));
    const results = await Promise.all(
      uniquePaths.map((relative_path) => resolveImageAssetPreview({ relative_path })),
    );

    return results.reduce<Record<string, string>>((acc, item) => {
      acc[item.relative_path] = item.preview_url;
      return acc;
    }, {});
  }, []);

  return {
    resolveImagePreviews,
  };
}
