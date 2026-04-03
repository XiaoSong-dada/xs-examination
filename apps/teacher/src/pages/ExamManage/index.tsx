import { Button, Select, Table, Tag, message, Progress, Modal, List, Typography } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef, useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

import { useExamManage, type ExamManageTableItem } from "@/hooks/useExamManage";
import { useTableHeight } from "@/hooks/useTableHeight";

const deviceStatusColorMap: Record<string, string> = {
  待分配: "default",
  未连接: "default",
  正常: "green",
  异常: "red",
};

interface DistributeProgress {
  exam_id: string;
  completed: number;
  total: number;
  progress: number;
  message: string;
  timestamp: number;
}

/**
 * 教师端考试管理页面。
 */
export function ExamManagePage() {
  const {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    currentExamStatusLabel,
    tableData,
    tableLoading,
    distributePapers,
    startExam,
    endExam,
    distributing,
    starting,
    ending,
  } = useExamManage();

  const [progressVisible, setProgressVisible] = useState(false);
  const [currentProgress, setCurrentProgress] = useState<DistributeProgress>({
    exam_id: "",
    completed: 0,
    total: 0,
    progress: 0,
    message: "准备开始分发",
    timestamp: Date.now(),
  });
  const [progressHistory, setProgressHistory] = useState<string[]>([]);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  // 监听分发进度事件
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupListener = async () => {
      unlistenFn = await listen("distribute-progress", (event) => {
        const progress = event.payload as DistributeProgress;
        if (progress.exam_id === selectedExamId) {
          setCurrentProgress(progress);
          setProgressHistory(prev => [...prev, progress.message]);

          // 当分发开始时显示进度模态框
          if (progress.completed === 0 && progress.total > 0) {
            setProgressVisible(true);
          }

          // 当分发完成时关闭进度模态框
          if (progress.completed === progress.total && progress.total > 0) {
            setTimeout(() => {
              setProgressVisible(false);
              setProgressHistory([]);
            }, 1000);
          }
        }
      });
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [selectedExamId]);

  const columns: ColumnsType<ExamManageTableItem> = [
    {
      title: "学生姓名",
      dataIndex: "name",
      key: "name",
      width: 180,
    },
    {
      title: "学生设备 IP",
      dataIndex: "deviceIp",
      key: "deviceIp",
      width: 220,
    },
    {
      title: "设备状态",
      dataIndex: "deviceStatus",
      key: "deviceStatus",
      width: 180,
      render: (value: string) => (
        <Tag color={deviceStatusColorMap[value] ?? "default"}>{value}</Tag>
      ),
    },
  ];

  const handleDistribute = async () => {
    if (!selectedExamId) {
      message.error("未选择考试，无法分发试卷");
      return;
    }

    // 重置进度状态
    setCurrentProgress({
      exam_id: selectedExamId,
      completed: 0,
      total: 0,
      progress: 0,
      message: "准备开始分发",
      timestamp: Date.now(),
    });
    setProgressHistory([]);

    const result = await distributePapers();
    if (!result) {
      message.error("未选择考试，无法分发试卷");
      setProgressVisible(false);
      return;
    }

    if (result.total === 0) {
      message.warning("当前考试没有可分发的已分配学生设备");
      setProgressVisible(false);
      return;
    }

    if (result.success_count === result.total) {
      message.success(`试卷分发成功（${result.success_count}/${result.total}）`);
    } else {
      const firstFailed = result.results.find((item) => !item.success);
      const detail = firstFailed?.message?.trim();
      message.warning(
        detail
          ? `试卷分发部分成功（${result.success_count}/${result.total}）：${detail}`
          : `试卷分发部分成功（${result.success_count}/${result.total}）`,
      );
    }
  };

  const handleStartExam = async () => {
    const result = await startExam();
    if (!result) {
      message.error("未选择考试，无法开始考试");
      return;
    }

    if (result.sent_count === 0) {
      message.warning("未向任何学生设备发送开始考试指令");
      return;
    }

    message.success(`开始考试指令已发送（${result.sent_count}/${result.total_targets}）`);
  };

  const handleEndExam = async () => {
    const result = await endExam();
    if (!result) {
      message.error("未选择考试，无法结束考试");
      return;
    }

    if (result.total_targets === 0) {
      message.success("当前无在线考生，考试已结束");
      return;
    }

    if (result.acked_count === result.total_targets) {
      message.success(`考试结束成功（最终同步 ${result.acked_count}/${result.total_targets}）`);
      return;
    }

    message.warning(
      `考试结束未完全确认（发送 ${result.sent_count}/${result.total_targets}，确认 ${result.acked_count}/${result.total_targets}）`,
    );
  };

  return (
    <div className="space-y-4 h-full">
      <div
        ref={containerRef}
        className="bg-white rounded-lg border border-gray-200 p-4 h-full"
      >
        <div
          ref={toolbarRef}
          className="bg-white rounded-lg flex flex-wrap items-center gap-3 pb-4 w-full"
        >
          <Select
            className="w-full max-w-md"
            placeholder="请选择考试"
            value={selectedExamId}
            loading={examLoading}
            options={examOptions}
            onChange={setSelectedExamId}
          />
          <Button type="primary" loading={distributing} onClick={() => void handleDistribute()}>
            分发试卷
          </Button>
          <Button type="primary" ghost loading={starting} onClick={() => void handleStartExam()}>
            开始考试
          </Button>
          <Button danger loading={ending} onClick={() => void handleEndExam()}>
            结束考试
          </Button>
          <div className="text-gray-600">当前考试状态：{currentExamStatusLabel}</div>
        </div>

        <Table<ExamManageTableItem>
          rowKey="id"
          loading={tableLoading}
          dataSource={tableData}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>

      {/* 分发进度模态框 */}
      <Modal
        title="试卷分发进度"
        open={progressVisible}
        onCancel={() => setProgressVisible(false)}
        footer={null}
        width={600}
      >
        <div className="space-y-4">
          <div>
            <div className="flex justify-between mb-1">
              <span>总体进度</span>
              <span>{currentProgress.completed}/{currentProgress.total}</span>
            </div>
            <Progress
              percent={currentProgress.progress}
              status="active"
              strokeColor={{
                from: '#108ee9',
                to: '#87d068'
              }}
            />
          </div>

          <div>
            <Typography.Title level={5}>分发状态</Typography.Title>
            <List
              size="small"
              dataSource={progressHistory}
              renderItem={(item) => (
                <List.Item>
                  <Typography.Text>{item}</Typography.Text>
                </List.Item>
              )}
            />
          </div>
        </div>
      </Modal>
    </div>
  );
}
