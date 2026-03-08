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

export interface IExamEditor extends IExamCreate {
  id: string;
}

export interface UseExamListResult {
  loading: boolean;
  inputKeyword: string;
  appliedKeyword: string;
  setInputKeyword: (value: string) => void;
  search: () => void;
  reset: () => void;
  page: number;
  pageSize: number;
  setPage: (value: number) => void;
  setPageSize: (value: number) => void;
  total: number;
  dataSource: ExamListItem[];
  refresh: () => Promise<void>;
}

export interface ExamListItem {
  id: string;
  title: string;
  description?: string;
  status: string;
}

export interface QuestionListItem {
  id: string;
  exam_id: string;
  seq: number;
  type: string;
  content: string;
  options?: string;
  answer: string;
  score: number;
  explanation?: string;
}

export interface IStudentCreate {
  student_no: string;
  name: string;
  created_at?: number;
  updated_at?: number;
}

export interface IStudentEditor extends IStudentCreate {
  id: string;
}

export interface StudentListItem {
  id: string;
  student_no: string;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface UseStudentListResult {
  loading: boolean;
  inputKeyword: string;
  appliedKeyword: string;
  setInputKeyword: (value: string) => void;
  search: () => void;
  reset: () => void;
  page: number;
  pageSize: number;
  setPage: (value: number) => void;
  setPageSize: (value: number) => void;
  total: number;
  dataSource: StudentListItem[];
  refresh: () => Promise<void>;
}
