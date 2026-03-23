import { invoke } from "@tauri-apps/api/core";
import type { CurrentExamBundle } from "@/types/main";

export async function getCurrentExamBundle(): Promise<CurrentExamBundle> {
  return invoke<CurrentExamBundle>("get_current_exam_bundle");
}

export async function sendAnswerSync(
  examId: string,
  studentId: string,
  questionId: string,
  answer: string,
): Promise<string> {
  return invoke<string>("send_answer_sync", {
    examId,
    studentId,
    questionId,
    answer,
  });
}
