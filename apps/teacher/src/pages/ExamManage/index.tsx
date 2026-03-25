import { Button, Select, Table, Tag, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";

import { useExamManage, type ExamManageTableItem } from "@/hooks/useExamManage";
import { useTableHeight } from "@/hooks/useTableHeight";

const deviceStatusColorMap: Record<string, string> = {
  待分配: "default",
  未连接: "default",
  正常: "green",
  异常: "red",
};

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

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

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
    const result = await distributePapers();
    if (!result) {
      message.error("未选择考试，无法分发试卷");
      return;
    }

    if (result.total === 0) {
      message.warning("当前考试没有可分发的已分配学生设备");
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
    </div>
  );
}
