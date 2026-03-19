import { invoke } from "@tauri-apps/api/core";
import type { CurrentExamBundle } from "@/types/main";

export async function getCurrentExamBundle(): Promise<CurrentExamBundle> {
  return invoke<CurrentExamBundle>("get_current_exam_bundle");
}
