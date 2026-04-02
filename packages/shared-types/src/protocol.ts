export type MessageType =
  | "EXAM_START"
  | "EXAM_PAUSE"
  | "EXAM_END"
  | "FORCE_SUBMIT"
  | "HEARTBEAT"
  | "ANSWER_SYNC"
  | "SUBMIT"
  | "STATUS_UPDATE"
  | "CHEAT_ALERT"
  | "P2P_DISTRIBUTION_START"
  | "P2P_DISTRIBUTION_PROGRESS"
  | "P2P_DISTRIBUTION_COMPLETE"
  | "P2P_DATA_BLOCK"
  | "P2P_BLOCK_REQUEST"
  | "P2P_BLOCK_RESPONSE";

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

// P2P 数据块结构
export interface DataBlock {
  blockId: string;
  examId: string;
  index: number;
  totalBlocks: number;
  data: string; // 用 base64 编码的二进制数据
  checksum: string; // 数据块的校验和
}

// P2P 分发开始通知
export interface P2pDistributionStartPayload {
  examId: string;
  totalBlocks: number;
  totalSize: number;
  blockSize: number;
  sourceDeviceId: string;
  timestamp: number;
}

// P2P 分发进度报告
export interface P2pDistributionProgressPayload {
  examId: string;
  deviceId: string;
  receivedBlocks: number;
  totalBlocks: number;
  progress: number;
  timestamp: number;
}

// P2P 分发完成通知
export interface P2pDistributionCompletePayload {
  examId: string;
  deviceId: string;
  success: boolean;
  message: string;
  completedAt: number;
}

// P2P 块请求
export interface P2pBlockRequestPayload {
  examId: string;
  blockIds: string[];
  requesterDeviceId: string;
  timestamp: number;
}

// P2P 块响应
export interface P2pBlockResponsePayload {
  examId: string;
  responderDeviceId: string;
  blocks: DataBlock[];
  timestamp: number;
}

// P2P 数据块消息
export interface P2pDataBlockPayload {
  sourceDeviceId: string;
  block: DataBlock;
  timestamp: number;
}
