import { invoke } from "@tauri-apps/api/core";
import { Question } from "@/types/main";

interface GetQuestionsPayload {
  exam_id: string;
}

interface BulkImportQuestionsPayload {
  exam_id: string;
  questions: Question[];
}

/**
 * 按考试查询题目列表。
 */
export async function getQuestionListByExamId(
  payload: GetQuestionsPayload,
): Promise<Question[]> {
  return invoke<Question[]>("get_questions", { payload });
}

/**
 * 批量导入题目（按考试覆盖导入）。
 */
export async function bulkImportQuestions(
  payload: BulkImportQuestionsPayload,
): Promise<Question[]> {
  return invoke<Question[]>("bulk_import_questions", { payload });
}
