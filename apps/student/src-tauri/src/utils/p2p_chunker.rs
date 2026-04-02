use sha2::{Sha256, Digest};
use hex::ToHex;

/// 默认块大小为 256KB
const DEFAULT_CHUNK_SIZE: usize = 256 * 1024;

/// 数据块信息结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkInfo {
    /// 块索引（从 0 开始）
    pub index: usize,
    /// 块的 SHA-256 哈希值（十六进制字符串）
    pub hash: String,
    /// 块的原始数据
    pub data: Vec<u8>,
}

/// 分块器配置
#[derive(Debug, Clone, Copy)]
pub struct ChunkerConfig {
    /// 块大小（字节）
    pub chunk_size: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        ChunkerConfig {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

/// 试卷分块与哈希计算器
pub struct P2PChunker {
    config: ChunkerConfig,
}

impl P2PChunker {
    /// 创建新的分块器，使用默认配置
    pub fn new() -> Self {
        P2PChunker {
            config: ChunkerConfig::default(),
        }
    }

    /// 使用自定义配置创建分块器
    pub fn with_config(config: ChunkerConfig) -> Self {
        P2PChunker { config }
    }

    /// 计算数据的 SHA-256 哈希值
    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.encode_hex::<String>()
    }

    /// 验证数据的哈希值是否匹配
    pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
        Self::compute_hash(data) == expected_hash
    }

    /// 将完整数据分割为固定大小的块
    pub fn split_data(&self, data: &[u8]) -> Vec<ChunkInfo> {
        let mut chunks = Vec::new();
        let chunk_size = self.config.chunk_size;
        
        for (index, chunk_data) in data.chunks(chunk_size).enumerate() {
            let hash = Self::compute_hash(chunk_data);
            chunks.push(ChunkInfo {
                index,
                hash,
                data: chunk_data.to_vec(),
            });
        }
        
        chunks
    }

    /// 从数据块还原完整数据，同时验证每个块的哈希
    pub fn reassemble_data(chunks: &[ChunkInfo]) -> Result<Vec<u8>, &'static str> {
        // 检查块是否按顺序排列
        let mut sorted_chunks: Vec<_> = chunks.iter().collect();
        sorted_chunks.sort_by_key(|chunk| chunk.index);
        
        // 验证索引连续性
        for (expected_idx, chunk) in sorted_chunks.iter().enumerate() {
            if chunk.index != expected_idx {
                return Err("Missing or out-of-order chunks");
            }
        }
        
        // 验证哈希并收集数据
        let mut result = Vec::new();
        for chunk in sorted_chunks {
            if !Self::verify_hash(&chunk.data, &chunk.hash) {
                return Err("Hash verification failed for chunk");
            }
            result.extend_from_slice(&chunk.data);
        }
        
        Ok(result)
    }
}

impl Default for P2PChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let data = b"Hello, world!";
        let hash = P2PChunker::compute_hash(data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 哈希是 64 个十六进制字符
    }

    #[test]
    fn test_hash_verification() {
        let data = b"Test data for verification";
        let hash = P2PChunker::compute_hash(data);
        assert!(P2PChunker::verify_hash(data, &hash));
        assert!(!P2PChunker::verify_hash(b"Wrong data", &hash));
    }

    #[test]
    fn test_split_small_data() {
        let chunker = P2PChunker::new();
        let data = b"This is a small test data";
        let chunks = chunker.split_data(data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(&chunks[0].data, data);
    }

    #[test]
    fn test_split_large_data() {
        let config = ChunkerConfig { chunk_size: 10 };
        let chunker = P2PChunker::with_config(config);
        
        // 创建 35 字节的数据
        let data: Vec<u8> = (0..35).collect();
        let chunks = chunker.split_data(&data);
        
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].data.len(), 10);
        assert_eq!(chunks[1].data.len(), 10);
        assert_eq!(chunks[2].data.len(), 10);
        assert_eq!(chunks[3].data.len(), 5);
        
        // 验证索引
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_reassemble_data() {
        let config = ChunkerConfig { chunk_size: 10 };
        let chunker = P2PChunker::with_config(config);
        
        let original_data: Vec<u8> = (0..35).collect();
        let chunks = chunker.split_data(&original_data);
        
        let reassembled = P2PChunker::reassemble_data(&chunks)
            .expect("Reassembly failed");
        
        assert_eq!(reassembled, original_data);
    }

    #[test]
    fn test_reassemble_out_of_order() {
        let config = ChunkerConfig { chunk_size: 10 };
        let chunker = P2PChunker::with_config(config);
        
        let original_data: Vec<u8> = (0..35).collect();
        let mut chunks = chunker.split_data(&original_data);
        
        // 打乱顺序
        chunks.swap(0, 1);
        
        let reassembled = P2PChunker::reassemble_data(&chunks)
            .expect("Reassembly failed even with out-of-order chunks");
        
        assert_eq!(reassembled, original_data);
    }

    #[test]
    fn test_reassemble_corrupted_chunk() {
        let config = ChunkerConfig { chunk_size: 10 };
        let chunker = P2PChunker::with_config(config);
        
        let original_data: Vec<u8> = (0..35).collect();
        let mut chunks = chunker.split_data(&original_data);
        
        // 破坏其中一个块的数据
        chunks[1].data[0] = 0xFF;
        
        let result = P2PChunker::reassemble_data(&chunks);
        assert!(result.is_err());
    }

    #[test]
    fn test_reassemble_missing_chunk() {
        let config = ChunkerConfig { chunk_size: 10 };
        let chunker = P2PChunker::with_config(config);
        
        let original_data: Vec<u8> = (0..35).collect();
        let mut chunks = chunker.split_data(&original_data);
        
        // 删除一个块
        chunks.remove(2);
        
        let result = P2PChunker::reassemble_data(&chunks);
        assert!(result.is_err());
    }
}
