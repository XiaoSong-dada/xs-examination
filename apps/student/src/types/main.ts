export interface Exam {
  id: string;
  title?: string;
  status?: string;
}

export interface ExamSession {
  id: string;
  examId: string;
  studentId: string;
  studentNo: string;
  studentName: string;
  assignedIpAddr: string;
  assignedDeviceName?: string;
  examTitle: string;
  status: string;
  assignmentStatus: string;
  startedAt?: number;
  endsAt?: number;
  paperVersion?: string;
  lastSyncedAt?: number;
  createdAt: number;
  updatedAt: number;
}

export interface ExamSnapshot {
  sessionId: string;
  examMeta: string;
  questionsPayload: string;
  downloadedAt: number;
  expiresAt?: number;
  updatedAt: number;
}

export interface CurrentExamBundle {
  session: ExamSession | null;
  snapshot: ExamSnapshot | null;
}

export interface RuntimeQuestion {
  id: string;
  seq: number;
  type: string;
  content: string;
  options: string[];
  score: number;
  explanation?: string;
  images: string[];
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
  currentSession: ExamSession | null;
  currentSnapshot: ExamSnapshot | null;
  questions: RuntimeQuestion[];
  loading: boolean;
  setCurrentExam: (exam: Exam | null) => void;
  refreshCurrentExam: () => Promise<void>;
}
