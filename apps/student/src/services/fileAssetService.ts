import { invoke } from "@tauri-apps/api/core";
import type {
  ResolveImageAssetPreviewPayload,
  ResolveImageAssetPreviewResult,
} from "@/types/fileAsset";

/**
 * 将学生端已持久化的图片相对路径解析为可渲染预览地址。
 *
 * @param payload 图片相对路径参数。
 * @returns 返回相对路径与可渲染预览 URL。
 */
export async function resolveImageAssetPreview(
  payload: ResolveImageAssetPreviewPayload,
): Promise<ResolveImageAssetPreviewResult> {
  return invoke<ResolveImageAssetPreviewResult>("resolve_image_asset_preview", { payload });
}
