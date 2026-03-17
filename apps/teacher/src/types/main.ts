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

/**
 * 题目实体结构（与数据库 questions 表字段保持一致）。
 */
export interface Question {
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

export type QuestionListItem = Question;

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

export interface StudentDeviceAssignItem {
  student_exam_id: string;
  student_id: string;
  student_no: string;
  student_name: string;
  ip_addr?: string;
  device_name?: string;
}

export interface StudentDeviceAssignPayloadItem {
  student_exam_id: string;
  ip_addr?: string;
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

export interface IDeviceCreate {
  ip: string;
  name: string;
}

export interface IDeviceEditor extends IDeviceCreate {
  id: string;
}

export interface DeviceListItem {
  id: string;
  ip: string;
  name: string;
}

export interface TeacherEndpointInput {
  id: string;
  endpoint: string;
  name?: string;
  remark?: string;
  isMaster: boolean;
}

export interface PushTeacherEndpointsPayload {
  deviceIds: string[];
  endpoints: TeacherEndpointInput[];
  controlPort?: number;
}

export interface PushTeacherEndpointsResultItem {
  deviceId: string;
  deviceIp: string;
  success: boolean;
  message: string;
  connectedMaster?: string;
}

export interface PushTeacherEndpointsResult {
  requestId: string;
  total: number;
  successCount: number;
  results: PushTeacherEndpointsResultItem[];
}

export interface UseDeviceListResult {
  loading: boolean;
  inputIpKeyword: string;
  inputNameKeyword: string;
  appliedIpKeyword: string;
  appliedNameKeyword: string;
  setInputIpKeyword: (value: string) => void;
  setInputNameKeyword: (value: string) => void;
  search: () => void;
  reset: () => void;
  dataSource: DeviceListItem[];
  refresh: () => Promise<void>;
  createDevice: (data: IDeviceCreate) => Promise<boolean>;
  updateDevice: (data: IDeviceEditor) => Promise<boolean>;
  deleteDevice: (id: string) => Promise<boolean>;
}


export interface DeviceAssignRow {
  id: string;
  student_exam_id: string;
  student_id: string;
  student_no: string;
  student_name: string;
  ip_addr?: string;
  device_name?: string;
  assigned: boolean;
}

export interface ExamOption {
  label: string;
  value: string;
}


export interface MonitorTableItem {
  id: string;
  name: string;
  deviceIp: string;
  linkStatus: string;
  answerProgress: number;
}
