import { useCallback } from "react";

import { resolveImageAssetPreview, uploadLocalImageAsset } from "@/services/fileAssetService";
import type { UploadLocalImageResult } from "@/types/fileAsset";

/**
 * 管理教师端本地文件上传到应用目录的前端调用逻辑。
 *
 * @returns 返回按业务目录批量上传本地图片的能力。
 */
export function useFileHooks() {
  /**
   * 批量上传本地图片到后端规范目录。
   *
   * @param paths - 用户选择的本地绝对路径列表。
   * @param biz - 业务目录标识，用于区分落盘子目录。
   * @returns 返回上传结果数组，每项包含相对路径和文件名。
   */
  const uploadLocalImages = useCallback(async (
    paths: string[],
    biz: string,
  ): Promise<UploadLocalImageResult[]> => {
    if (paths.length === 0) {
      return [];
    }

    return Promise.all(paths.map((source_path) => uploadLocalImageAsset({ source_path, biz })));
  }, []);

  /**
   * 上传题库题目相关图片到 question-bank 目录。
   *
   * @param paths - 本地图片绝对路径列表。
   * @param folder - 题库业务子目录，如 content 或 options。
   * @returns 返回上传结果数组。
   */
  const uploadQuestionBankImages = useCallback(async (
    paths: string[],
    folder: "content" | "options",
  ): Promise<UploadLocalImageResult[]> => {
    return uploadLocalImages(paths, `question-bank/${folder}`);
  }, [uploadLocalImages]);

  /**
   * 批量解析图片相对路径为可渲染的缩略图预览地址。
   *
   * @param relativePaths - 持久化存储的图片相对路径列表。
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
    uploadLocalImages,
    uploadQuestionBankImages,
    resolveImagePreviews,
  };
}
