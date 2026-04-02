use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    ExamStart,
    ExamPause,
    ExamEnd,
    FinalSyncRequest,
    PaperPackageManifest,
    PaperPackageChunk,
    PaperPackageAck,
    PaperAssetManifest,
    PaperAssetChunk,
    PaperAssetAck,
    PaperAssetSyncDone,
    ForceSubmit,
    Heartbeat,
    AnswerSync,
    AnswerSyncAck,
    Submit,
    StatusUpdate,
    CheatAlert,
    P2pDistributionStart,
    P2pDistributionProgress,
    P2pDistributionComplete,
    P2pDataBlock,
    P2pBlockRequest,
    P2pBlockResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPackageManifestPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    #[serde(rename = "totalChunks")]
    pub total_chunks: u32,
    pub sha256: String,
    #[serde(rename = "examTitle")]
    pub exam_title: String,
    #[serde(rename = "assignmentStatus")]
    pub assignment_status: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    #[serde(rename = "paperVersion")]
    pub paper_version: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPackageChunkPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "chunkIndex")]
    pub chunk_index: u32,
    #[serde(rename = "totalChunks")]
    pub total_chunks: u32,
    #[serde(rename = "contentBase64")]
    pub content_base64: String,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPackageAckPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    pub success: bool,
    pub message: String,
    #[serde(rename = "receivedChunks")]
    pub received_chunks: u32,
    #[serde(rename = "totalChunks")]
    pub total_chunks: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub r#type: MessageType,
    pub timestamp: i64,
    pub signature: String,
    pub payload: T,
}

pub fn build_message<T>(message_type: MessageType, timestamp: i64, payload: T) -> WsMessage<T> {
    WsMessage {
        r#type: message_type,
        timestamp,
        signature: String::new(),
        payload,
    }
}

pub fn encode_message<T: Serialize>(message: &WsMessage<T>) -> anyhow::Result<String> {
    Ok(serde_json::to_string(message)?)
}

pub fn decode_value_message(text: &str) -> anyhow::Result<WsMessage<Value>> {
    Ok(serde_json::from_str(text)?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    #[serde(rename = "studentId")]
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerItem {
    #[serde(rename = "questionId")]
    pub question_id: String,
    pub answer: String,
    #[serde(default)]
    pub revision: Option<i64>,
    #[serde(rename = "answerUpdatedAt", default)]
    pub answer_updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "syncMode", default)]
    pub sync_mode: Option<String>,
    #[serde(rename = "batchId", default)]
    pub batch_id: Option<String>,
    pub answers: Vec<AnswerItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncAckPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "syncMode", default)]
    pub sync_mode: Option<String>,
    #[serde(rename = "batchId", default)]
    pub batch_id: Option<String>,
    pub success: bool,
    pub message: String,
    #[serde(rename = "ackedAt")]
    pub acked_at: i64,
    #[serde(rename = "questionIds", default)]
    pub question_ids: Vec<String>,
    #[serde(rename = "failedQuestionIds", default)]
    pub failed_question_ids: Vec<String>,
    #[serde(rename = "successCount", default)]
    pub success_count: i64,
    #[serde(rename = "failedCount", default)]
    pub failed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamStartPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalSyncRequestPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamEndPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "endTime")]
    pub end_time: i64,
    #[serde(rename = "finalBatchId")]
    pub final_batch_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAssetDescriptor {
    #[serde(rename = "assetId")]
    pub asset_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub sha256: String,
    #[serde(rename = "byteSize")]
    pub byte_size: u64,
    #[serde(rename = "localPath", default)]
    pub local_path: Option<String>,
    #[serde(rename = "relativePath", default)]
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAssetManifestPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    pub assets: Vec<PaperAssetDescriptor>,
    #[serde(rename = "totalAssets")]
    pub total_assets: usize,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAssetChunkPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "assetId")]
    pub asset_id: String,
    #[serde(rename = "chunkIndex")]
    pub chunk_index: u32,
    #[serde(rename = "totalChunks")]
    pub total_chunks: u32,
    #[serde(rename = "contentBase64")]
    pub content_base64: String,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAssetAckPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "assetId")]
    pub asset_id: String,
    pub success: bool,
    pub message: String,
    #[serde(rename = "receivedChunks", default)]
    pub received_chunks: Option<u32>,
    #[serde(rename = "totalChunks", default)]
    pub total_chunks: Option<u32>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAssetSyncDonePayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    pub success: bool,
    pub message: String,
    #[serde(rename = "totalAssets")]
    pub total_assets: usize,
    #[serde(rename = "successAssets")]
    pub success_assets: usize,
    #[serde(rename = "failedAssetIds", default)]
    pub failed_asset_ids: Vec<String>,
    pub timestamp: i64,
}

// P2P 数据块结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBlock {
    #[serde(rename = "blockId")]
    pub block_id: String,
    #[serde(rename = "examId")]
    pub exam_id: String,
    pub index: u64,
    #[serde(rename = "totalBlocks")]
    pub total_blocks: u64,
    pub data: String,
    pub checksum: String,
}

// P2P 分发开始通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pDistributionStartPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "totalBlocks")]
    pub total_blocks: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    #[serde(rename = "blockSize")]
    pub block_size: u64,
    #[serde(rename = "sourceDeviceId")]
    pub source_device_id: String,
    pub timestamp: i64,
}

// P2P 分发进度报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pDistributionProgressPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "receivedBlocks")]
    pub received_blocks: u64,
    #[serde(rename = "totalBlocks")]
    pub total_blocks: u64,
    pub progress: f64,
    pub timestamp: i64,
}

// P2P 分发完成通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pDistributionCompletePayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub success: bool,
    pub message: String,
    #[serde(rename = "completedAt")]
    pub completed_at: i64,
}

// P2P 块请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pBlockRequestPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "blockIds")]
    pub block_ids: Vec<String>,
    #[serde(rename = "requesterDeviceId")]
    pub requester_device_id: String,
    pub timestamp: i64,
}

// P2P 块响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pBlockResponsePayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "responderDeviceId")]
    pub responder_device_id: String,
    pub blocks: Vec<DataBlock>,
    pub timestamp: i64,
}

// P2P 数据块消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pDataBlockPayload {
    #[serde(rename = "sourceDeviceId")]
    pub source_device_id: String,
    pub block: DataBlock,
    pub timestamp: i64,
}
