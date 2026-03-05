import type { Dayjs } from "@/utils/dayjs";

/**
 * 表单提交用的考试创建类型（用于 `ExamCreate` 页面表单）
 */
export interface IExamCreate {
  title: string;
  description?: string;
  // 表单中可接收 Dayjs 对象或数字时间戳，提交到后端时会转换为毫秒数或 null
  start_time?: Dayjs | number | null;
  end_time?: Dayjs | number | null;
  pass_score?: number;
  status?: string;
  // 表单 Switch 输出为 boolean，后端存储为 0/1
  shuffle_questions?: boolean | number;
  shuffle_options?: boolean | number;
}
