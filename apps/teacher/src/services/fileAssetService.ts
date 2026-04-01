import { invoke } from "@tauri-apps/api/core";
import type { UploadLocalImagePayload, UploadLocalImageResult } from "@/types/fileAsset";

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
