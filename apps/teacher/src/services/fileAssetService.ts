import { invoke } from "@tauri-apps/api/core";
import type {
  ResolveImageAssetPreviewPayload,
  ResolveImageAssetPreviewResult,
  UploadLocalImagePayload,
  UploadLocalImageResult,
} from "@/types/fileAsset";

/**
 * 调用教师端后端命令，将本地图片复制到应用规范目录。
 *
 * @param payload - 上传参数，包含源文件路径和业务目录标识。
 * @returns 返回图片相对路径与最终文件名。
 */
export async function uploadLocalImageAsset(
  payload: UploadLocalImagePayload,
): Promise<UploadLocalImageResult> {
  return invoke<UploadLocalImageResult>("upload_local_image_asset", { payload });
}

/**
 * 调用教师端后端命令，将已保存的图片相对路径转换为可展示预览的 data URL。
 *
 * @param payload - 预览参数，包含图片相对路径。
 * @returns 返回图片相对路径及对应预览地址。
 */
export async function resolveImageAssetPreview(
  payload: ResolveImageAssetPreviewPayload,
): Promise<ResolveImageAssetPreviewResult> {
  return invoke<ResolveImageAssetPreviewResult>("resolve_image_asset_preview", { payload });
}
