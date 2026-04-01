import { invoke } from "@tauri-apps/api/core";
import type {
  IQuestionBankCreate,
  IQuestionBankEditor,
  Question,
  QuestionBankExportPackageResult,
  QuestionBankItem,
} from "@/types/main";

interface GetQuestionsPayload {
  exam_id: string;
}

interface BulkImportQuestionsPayload {
  exam_id: string;
  questions: Question[];
}

interface GetQuestionBankItemByIdPayload {
  id: string;
}

interface ExportQuestionBankPackagePayload {
  file_name: string;
  xlsx_bytes: number[];
  image_relative_paths: string[];
}

interface ImportQuestionPackagePayload {
  exam_id: string;
  package_path: string;
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

/**
 * 查询全局题库题目列表。
 *
 * @returns 返回教师端独立题库列表。
 */
export async function getQuestionBankItems(): Promise<QuestionBankItem[]> {
  return invoke<QuestionBankItem[]>("get_question_bank_items");
}

/**
 * 按 ID 查询单条全局题库题目。
 *
 * @param payload - 包含题目 ID 的查询参数。
 * @returns 返回对应题目详情。
 */
export async function getQuestionBankItemById(
  payload: GetQuestionBankItemByIdPayload,
): Promise<QuestionBankItem> {
  return invoke<QuestionBankItem>("get_question_bank_item_by_id", { payload });
}

/**
 * 新增一条全局题库题目。
 *
 * @param payload - 题目表单数据。
 * @returns 返回新建后的题目详情。
 */
export async function createQuestionBankItem(
  payload: IQuestionBankCreate,
): Promise<QuestionBankItem> {
  return invoke<QuestionBankItem>("create_question_bank_item", { payload });
}

/**
 * 更新一条全局题库题目。
 *
 * @param payload - 包含题目 ID 与最新字段的表单数据。
 * @returns 返回更新后的题目详情。
 */
export async function updateQuestionBankItem(
  payload: IQuestionBankEditor,
): Promise<QuestionBankItem> {
  return invoke<QuestionBankItem>("update_question_bank_item", { payload });
}

/**
 * 删除一条全局题库题目。
 *
 * @param id - 题目 ID。
 * @returns 删除成功时返回空结果。
 */
export async function deleteQuestionBankItem(id: string): Promise<void> {
  return invoke<void>("delete_question_bank_item", { payload: { id } });
}

/**
 * 导出题库资源包（question_bank.xlsx + assets）。
 *
 * @param payload - 导出文件名、xlsx 字节与图片相对路径。
 * @returns 返回导出后的 zip 保存结果。
 */
export async function exportQuestionBankPackage(
  payload: ExportQuestionBankPackagePayload,
): Promise<QuestionBankExportPackageResult> {
  return invoke<QuestionBankExportPackageResult>("export_question_bank_package", {
    payload,
  });
}

/**
 * 按考试导入题目资源包并覆盖题目列表。
 *
 * @param payload - 考试 ID 与资源包绝对路径。
 * @returns 返回导入后的题目列表。
 */
export async function importQuestionPackageByExamId(
  payload: ImportQuestionPackagePayload,
): Promise<Question[]> {
  return invoke<Question[]>("import_question_package_by_exam_id", { payload });
}
