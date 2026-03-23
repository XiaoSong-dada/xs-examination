import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CurrentExamBundle, LocalAnswer } from "@/types/main";

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

export async function getCurrentSessionAnswers(): Promise<LocalAnswer[]> {
  return invoke<LocalAnswer[]>("get_current_session_answers");
}

export async function onExamStatusChanged(
  handler: () => void,
): Promise<UnlistenFn> {
  return listen("exam_status_changed", () => {
    handler();
  });
}
