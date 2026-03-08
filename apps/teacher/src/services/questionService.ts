import type { QuestionListItem } from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

interface GetQuestionsPayload {
  exam_id: string;
}

/**
 * 按考试查询题目列表。
 */
export async function getQuestionListByExamId(
  payload: GetQuestionsPayload,
): Promise<QuestionListItem[]> {
  return invoke<QuestionListItem[]>("get_questions", { payload });
}
