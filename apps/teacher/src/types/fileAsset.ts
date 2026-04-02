export interface UploadLocalImagePayload {
  source_path: string;
  biz: string;
}

export interface UploadLocalImageResult {
  relative_path: string;
  file_name: string;
}

export interface ResolveImageAssetPreviewPayload {
  relative_path: string;
}

export interface ResolveImageAssetPreviewResult {
  relative_path: string;
  preview_url: string;
}
