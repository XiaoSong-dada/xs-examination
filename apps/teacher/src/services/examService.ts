import type { IExamCreate, IExamEditor } from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

export interface ExamListItem {
  id: string;
  title: string;
  description?: string;
  status: string;
}

export interface ExamDetailItem extends IExamEditor {
  pass_score: number;
  status: string;
  shuffle_questions: number;
  shuffle_options: number;
}


/**
 * 创建考试
 *
 * @param data - 考试创建表单数据
 * @returns 创建成功后的完整考试对象
 */
export async function createExam(data: IExamCreate) {
  return invoke("create_exam", { payload: data });
}

/**
 * 更新考试
 *
 * @param data - 考试编辑表单数据（含 id）
 */
export async function updateExam(data: IExamEditor) {
  return invoke("update_exam", { payload: data });
}

/**
 * 删除考试
 *
 * @param id - 考试 id
 */
export async function deleteExam(id: string) {
  return invoke("delete_exam", { payload: { id } });
}

/**
 * 获取单个考试详情
 *
 * @param id - 考试 id
 */
export async function getExamById(id: string): Promise<ExamDetailItem> {
  return invoke<ExamDetailItem>("get_exam_by_id", { payload: { id } });
}

/**
 * 教师端考试模块 IPC 进程通信层。
 *
 * 该层只负责与 Tauri Rust 命令交互，不承载页面状态与业务组合逻辑。
 */

/**
 * 通过 Tauri IPC 获取考试列表。
 *
 * @returns 返回教师端当前所有考试列表（按创建时间倒序）。
 */
export async function getExamList(): Promise<ExamListItem[]> {
  return invoke<ExamListItem[]>("get_exams");
}
