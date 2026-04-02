use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::network::protocol::P2pDistributionProgressPayload;
use crate::utils::p2p_chunker::{ChunkInfo, P2PChunker};

/// 设备分发状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDistributionStatus {
    /// 设备 ID
    pub device_id: String,
    /// 设备 IP 地址
    pub device_ip: String,
    /// 设备控制端口
    pub control_port: u16,
    /// 已分配的块索引集合
    pub assigned_blocks: HashSet<usize>,
    /// 已接收的块索引集合
    pub received_blocks: HashSet<usize>,
    /// 当前进度（0.0-1.0）
    pub progress: f64,
    /// 分发是否成功完成
    pub success: bool,
    /// 状态消息
    pub message: String,
}

impl DeviceDistributionStatus {
    pub fn new(device_id: String, device_ip: String, control_port: u16) -> Self {
        DeviceDistributionStatus {
            device_id,
            device_ip,
            control_port,
            assigned_blocks: HashSet::new(),
            received_blocks: HashSet::new(),
            progress: 0.0,
            success: false,
            message: String::from("Pending"),
        }
    }
}

/// 分发任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionTaskStatus {
    /// 待开始
    Pending,
    /// 正在分发
    Distributing,
    /// 分发完成
    Completed,
    /// 分发失败
    Failed,
}

/// 分发任务信息
#[derive(Debug, Clone)]
pub struct DistributionTask {
    /// 任务 ID
    pub task_id: String,
    /// 考试 ID
    pub exam_id: String,
    /// 总块数
    pub total_blocks: usize,
    /// 所有数据块
    pub chunks: Vec<ChunkInfo>,
    /// 设备分发状态映射
    pub device_statuses: HashMap<String, DeviceDistributionStatus>,
    /// 任务状态
    pub status: DistributionTaskStatus,
    /// 开始时间戳
    pub start_time: Option<i64>,
    /// 完成时间戳
    pub complete_time: Option<i64>,
}

impl DistributionTask {
    pub fn new(task_id: String, exam_id: String, chunks: Vec<ChunkInfo>) -> Self {
        DistributionTask {
            task_id,
            exam_id,
            total_blocks: chunks.len(),
            chunks,
            device_statuses: HashMap::new(),
            status: DistributionTaskStatus::Pending,
            start_time: None,
            complete_time: None,
        }
    }

    /// 计算整体分发进度
    pub fn overall_progress(&self) -> f64 {
        if self.device_statuses.is_empty() {
            return 0.0;
        }

        let total_received: usize = self
            .device_statuses
            .values()
            .map(|status| status.received_blocks.len())
            .sum();
        let total_required = self.total_blocks * self.device_statuses.len();

        if total_required == 0 {
            return 0.0;
        }

        total_received as f64 / total_required as f64
    }

    /// 检查分发是否完成（所有设备都收到了所有块）
    pub fn is_complete(&self) -> bool {
        if self.device_statuses.is_empty() {
            return false;
        }

        self.device_statuses.values().all(|status| {
            status.received_blocks.len() == self.total_blocks
        })
    }

    /// 检查分发是否成功
    pub fn is_success(&self) -> bool {
        self.device_statuses.values().all(|status| status.success)
    }
}

/// 块分配策略 trait
pub trait ChunkAllocationStrategy {
    /// 为设备分配块
    fn allocate_chunks(
        &self,
        chunks: &[ChunkInfo],
        devices: &[DeviceDistributionStatus],
    ) -> HashMap<String, HashSet<usize>>;
}

/// 轮询分配策略
pub struct RoundRobinAllocation;

impl RoundRobinAllocation {
    pub fn new() -> Self {
        RoundRobinAllocation
    }
}

impl Default for RoundRobinAllocation {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkAllocationStrategy for RoundRobinAllocation {
    fn allocate_chunks(
        &self,
        chunks: &[ChunkInfo],
        devices: &[DeviceDistributionStatus],
    ) -> HashMap<String, HashSet<usize>> {
        let mut result = HashMap::new();
        if chunks.is_empty() || devices.is_empty() {
            return result;
        }

        // 初始化每个设备的分配集合
        for device in devices {
            result.insert(device.device_id.clone(), HashSet::new());
        }

        // 轮询分配块
        for (chunk_idx, _) in chunks.iter().enumerate() {
            let device_idx = chunk_idx % devices.len();
            let device_id = devices[device_idx].device_id.clone();
            if let Some(blocks) = result.get_mut(&device_id) {
                blocks.insert(chunk_idx);
            }
        }

        result
    }
}

/// P2P 分发协调器
pub struct P2PDistributor {
    /// 正在进行的分发任务（task_id -> Task）
    tasks: Arc<Mutex<HashMap<String, DistributionTask>>>,
    /// 块分配策略
    allocation_strategy: Box<dyn ChunkAllocationStrategy + Send + Sync>,
}

impl P2PDistributor {
    /// 创建新的 P2P 分发协调器
    pub fn new() -> Self {
        P2PDistributor {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            allocation_strategy: Box::new(RoundRobinAllocation::new()),
        }
    }

    /// 使用自定义策略创建协调器
    pub fn with_strategy<S>(strategy: S) -> Self
    where
        S: ChunkAllocationStrategy + Send + Sync + 'static,
    {
        P2PDistributor {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            allocation_strategy: Box::new(strategy),
        }
    }

    /// 创建新的分发任务
    pub fn create_task(
        &self,
        task_id: String,
        exam_id: String,
        data: &[u8],
    ) -> Result<String> {
        let chunker = P2PChunker::new();
        let chunks = chunker.split_data(data);

        let task = DistributionTask::new(task_id.clone(), exam_id, chunks);

        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        tasks.insert(task_id.clone(), task);

        Ok(task_id)
    }

    /// 为任务添加设备
    pub fn add_device_to_task(
        &self,
        task_id: &str,
        device_id: String,
        device_ip: String,
        control_port: u16,
    ) -> Result<()> {
        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        let status = DeviceDistributionStatus::new(device_id.clone(), device_ip, control_port);
        task.device_statuses.insert(device_id, status);

        Ok(())
    }

    /// 开始分发任务
    pub fn start_distribution(&self, task_id: &str) -> Result<HashMap<String, HashSet<usize>>> {
        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        // 准备设备列表
        let devices: Vec<_> = task.device_statuses.values().cloned().collect();

        // 使用策略分配块
        let allocation = self
            .allocation_strategy
            .allocate_chunks(&task.chunks, &devices);

        // 更新每个设备的分配状态
        for (device_id, block_indices) in &allocation {
            if let Some(status) = task.device_statuses.get_mut(device_id) {
                status.assigned_blocks = block_indices.clone();
            }
        }

        // 更新任务状态
        task.status = DistributionTaskStatus::Distributing;
        task.start_time = Some(chrono::Utc::now().timestamp());

        Ok(allocation)
    }

    /// 更新设备分发进度
    pub fn update_device_progress(
        &self,
        task_id: &str,
        progress: P2pDistributionProgressPayload,
    ) -> Result<()> {
        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        let status = task
            .device_statuses
            .get_mut(&progress.device_id)
            .ok_or_else(|| anyhow!("Device not found: {}", progress.device_id))?;

        // 更新进度
        status.progress = progress.progress;

        // 如果进度是 1.0，标记所有块已接收
        if status.progress >= 1.0 {
            status.received_blocks = (0..task.total_blocks).collect();
            status.success = true;
            status.message = String::from("Completed successfully");
        }

        // 检查任务是否全部完成
        if task.is_complete() {
            task.status = DistributionTaskStatus::Completed;
            task.complete_time = Some(chrono::Utc::now().timestamp());
        }

        Ok(())
    }

    /// 记录设备成功接收到块
    pub fn mark_block_received(
        &self,
        task_id: &str,
        device_id: &str,
        block_index: usize,
    ) -> Result<()> {
        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        let status = task
            .device_statuses
            .get_mut(device_id)
            .ok_or_else(|| anyhow!("Device not found: {}", device_id))?;

        status.received_blocks.insert(block_index);
        status.progress = status.received_blocks.len() as f64 / task.total_blocks as f64;

        if status.received_blocks.len() == task.total_blocks {
            status.success = true;
            status.message = String::from("Completed successfully");
        }

        // 检查任务是否全部完成
        if task.is_complete() {
            task.status = DistributionTaskStatus::Completed;
            task.complete_time = Some(chrono::Utc::now().timestamp());
        }

        Ok(())
    }

    /// 获取任务信息
    pub fn get_task(&self, task_id: &str) -> Result<DistributionTask> {
        let tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        tasks
            .get(task_id)
            .cloned()
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))
    }

    /// 获取任务的整体进度
    pub fn get_task_progress(&self, task_id: &str) -> Result<f64> {
        let tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        Ok(task.overall_progress())
    }

    /// 获取数据块
    pub fn get_chunk(&self, task_id: &str, index: usize) -> Result<ChunkInfo> {
        let tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        task.chunks
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow!("Chunk not found at index: {}", index))
    }

    /// 获取所有数据块
    pub fn get_all_chunks(&self, task_id: &str) -> Result<Vec<ChunkInfo>> {
        let tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let task = tasks
            .get(task_id)
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        Ok(task.chunks.clone())
    }

    /// 删除任务
    pub fn remove_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        tasks.remove(task_id);
        Ok(())
    }
}

impl Default for P2PDistributor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin_allocation() {
        let strategy = RoundRobinAllocation::new();

        // 创建一些测试块
        let chunker = P2PChunker::with_config(crate::utils::p2p_chunker::ChunkerConfig {
            chunk_size: 10,
        });
        let data: Vec<u8> = (0..35).collect();
        let chunks = chunker.split_data(&data);

        // 创建一些测试设备
        let devices = vec![
            DeviceDistributionStatus::new(
                "device1".to_string(),
                "192.168.1.1".to_string(),
                8080,
            ),
            DeviceDistributionStatus::new(
                "device2".to_string(),
                "192.168.1.2".to_string(),
                8080,
            ),
        ];

        let allocation = strategy.allocate_chunks(&chunks, &devices);

        // 验证分配结果
        assert_eq!(allocation.len(), 2);
        assert!(allocation.contains_key("device1"));
        assert!(allocation.contains_key("device2"));

        // device1 应该有索引 0, 2
        let device1_blocks = allocation.get("device1").unwrap();
        assert!(device1_blocks.contains(&0));
        assert!(device1_blocks.contains(&2));

        // device2 应该有索引 1, 3
        let device2_blocks = allocation.get("device2").unwrap();
        assert!(device2_blocks.contains(&1));
        assert!(device2_blocks.contains(&3));
    }

    #[test]
    fn test_distributor_creation() {
        let distributor = P2PDistributor::new();

        // 创建测试任务
        let data = b"Test exam data for P2P distribution";
        let task_id = distributor
            .create_task("task-1".to_string(), "exam-1".to_string(), data)
            .expect("Failed to create task");

        assert_eq!(task_id, "task-1");

        // 添加设备
        distributor
            .add_device_to_task("task-1", "device-1".to_string(), "192.168.1.1".to_string(), 8080)
            .expect("Failed to add device");

        // 开始分发
        let allocation = distributor
            .start_distribution("task-1")
            .expect("Failed to start distribution");

        assert!(!allocation.is_empty());

        // 获取任务
        let task = distributor.get_task("task-1").expect("Failed to get task");
        assert_eq!(task.status, DistributionTaskStatus::Distributing);
    }
}
