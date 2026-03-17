export interface Exam {
  id: string;
  title?: string;
  status?: string;
}

export interface AssignedStudent {
  studentNo: string;
  name: string;
}

export type TeacherConnectionStatus =
  | "connected"
  | "disconnected"
  | "connecting"
  | "unknown";

export interface TeacherRuntimeStatus {
  endpoint: string | null;
  connectionStatus: TeacherConnectionStatus;
}

export interface TeacherEndpointAppliedEvent {
  endpoint: string | null;
}

export interface WsConnectionEvent {
  endpoint: string | null;
  connected: boolean;
  message?: string | null;
}

export interface DeviceStore {
  ip: string | null;
  assignedStudent: AssignedStudent | null;
  teacherMasterEndpoint: string | null;
  teacherConnectionStatus: TeacherConnectionStatus;
  setIp: (ip: string | null) => void;
  setAssignedStudent: (s: AssignedStudent | null) => void;
  setTeacherMasterEndpoint: (ep: string | null) => void;
  setTeacherConnectionStatus: (s: TeacherConnectionStatus) => void;
  initTeacherInfo: () => Promise<void>;
}

export interface ExamStore {
  currentExam: Exam | null;
  setCurrentExam: (exam: Exam | null) => void;
}
