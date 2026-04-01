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

export type QuestionBankOptionType = "text" | "text_with_image";

export interface QuestionBankOption {
  key: string;
  text: string;
  option_type: QuestionBankOptionType;
  image_paths: string[];
}

export interface QuestionBankItem {
  id: string;
  type: string;
  content: string;
  content_image_paths: string[];
  options: QuestionBankOption[];
  answer: string;
  score: number;
  explanation?: string;
  created_at: number;
  updated_at: number;
}

export interface IQuestionBankCreate {
  type: string;
  content: string;
  content_image_paths: string[];
  options: QuestionBankOption[];
  answer: string;
  score: number;
  explanation?: string;
  created_at?: number;
  updated_at?: number;
}

export interface IQuestionBankEditor extends IQuestionBankCreate {
  id: string;
}

export interface QuestionBankExportPackageResult {
  path: string;
  packed_image_count: number;
  missing_image_count: number;
}

export interface UseQuestionBankListResult {
  loading: boolean;
  inputKeyword: string;
  appliedKeyword: string;
  setInputKeyword: (value: string) => void;
  typeFilter?: string;
  appliedTypeFilter?: string;
  setTypeFilter: (value?: string) => void;
  search: () => void;
  reset: () => void;
  total: number;
  dataSource: QuestionBankItem[];
  refresh: () => Promise<void>;
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

export type DeviceConnectionStatus = "待分配" | "未连接" | "正常" | "异常";

export interface StudentDeviceConnectionStatusItem {
  student_exam_id: string;
  student_id: string;
  student_no: string;
  student_name: string;
  ip_addr?: string;
  device_name?: string;
  connection_status: DeviceConnectionStatus;
  last_heartbeat_at?: number;
  has_heartbeat_seen: boolean;
  answered_count: number;
  total_questions: number;
  progress_percent: number;
}

export interface StudentScoreSummaryItem {
  student_id: string;
  total_score: number;
  is_passed: boolean;
  graded_at: number;
}

export interface DistributeExamPapersResultItem {
  student_exam_id: string;
  student_id: string;
  device_ip: string;
  success: boolean;
  message: string;
  session_id?: string;
}

export interface DistributeExamPapersResult {
  request_id: string;
  total: number;
  success_count: number;
  results: DistributeExamPapersResultItem[];
}

export interface StartExamResult {
  exam_id: string;
  total_targets: number;
  sent_count: number;
}

export interface EndExamResult {
  request_id: string;
  exam_id: string;
  total_targets: number;
  sent_count: number;
  acked_count: number;
  failed_count: number;
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
  connection_status: DeviceConnectionStatus;
  last_heartbeat_at?: number;
  has_heartbeat_seen: boolean;
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
