export interface Exam {
  id: string;
  title?: string;
  status?: string;
}

export interface AssignedStudent {
  studentNo: string;
  name: string;
}


export interface DeviceStore {
  ip: string | null;
  assignedStudent: AssignedStudent | null;
  setIp: (ip: string | null) => void;
  setAssignedStudent: (s: AssignedStudent | null) => void;
}

export interface ExamStore {
  currentExam: Exam | null;
  setCurrentExam: (exam: Exam | null) => void;
}
