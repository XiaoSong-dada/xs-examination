import { open } from "@tauri-apps/plugin-dialog";

const imageExtensions = ["png", "jpg", "jpeg", "gif", "webp", "bmp"];

/**
 * 打开教师端本地图片选择器并返回所选文件路径。
 *
 * @param multiple - 是否允许多选图片。
 * @returns 返回选中的图片绝对路径数组；取消选择时返回空数组。
 */
export async function pickImageFilePaths(multiple = true): Promise<string[]> {
  const result = await open({
    directory: false,
    multiple,
    filters: [
      {
        name: "Images",
        extensions: imageExtensions,
      },
    ],
  });

  if (!result) {
    return [];
  }

  return Array.isArray(result) ? result : [result];
}