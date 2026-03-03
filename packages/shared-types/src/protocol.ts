export type MessageType =
  | "EXAM_START"
  | "EXAM_PAUSE"
  | "EXAM_END"
  | "FORCE_SUBMIT"
  | "HEARTBEAT"
  | "ANSWER_SYNC"
  | "SUBMIT"
  | "STATUS_UPDATE"
  | "CHEAT_ALERT";

export interface WsMessage<T = unknown> {
  type: MessageType;
  timestamp: number;
  signature: string;
  payload: T;
}

export interface AnswerSyncPayload {
  examId: string;
  studentId: string;
  answers: { questionId: string; answer: string }[];
}

export interface StatusUpdatePayload {
  studentId: string;
  progress: number;
  currentQuestion: number;
}
