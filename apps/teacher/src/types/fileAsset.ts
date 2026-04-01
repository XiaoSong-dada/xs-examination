export interface UploadLocalImagePayload {
  source_path: string;
  biz: string;
}

export interface UploadLocalImageResult {
  relative_path: string;
  file_name: string;
}
