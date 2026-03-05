import { IExamCreate } from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

export interface ExamListItem {
  id: string;
  title: string;
  status: string;
}


/**
 * 创建考试
 *
 * @param data - 考试创建表单数据
 * @returns 创建成功后的完整考试对象
 */
export async function createExam(data: IExamCreate) {
  return invoke('create_exam', { data });
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
