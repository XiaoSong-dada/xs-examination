import { Button, Select, Table, Tag, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";

import { useExamManage, type ExamManageTableItem } from "@/hooks/useExamManage";
import { useTableHeight } from "@/hooks/useTableHeight";

const linkStatusColorMap: Record<string, string> = {
  已连接: "green",
  未连接: "default",
};

const examStudentStatusColorMap: Record<string, string> = {
  待考: "default",
  作答中: "processing",
  已交卷: "blue",
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
    distributing,
    starting,
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
      title: "链接状态",
      dataIndex: "linkStatus",
      key: "linkStatus",
      width: 180,
      render: (value: string) => (
        <Tag color={linkStatusColorMap[value] ?? "default"}>{value}</Tag>
      ),
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
      width: 160,
      render: (value: string) => (
        <Tag color={examStudentStatusColorMap[value] ?? "default"}>{value}</Tag>
      ),
    },
  ];

  const handleDistribute = async () => {
    const ok = await distributePapers();
    if (ok) {
      message.success("试卷分发成功");
    } else {
      message.error("试卷分发失败");
    }
  };

  const handleStartExam = async () => {
    const ok = await startExam();
    if (ok) {
      message.success("考试已开始");
    } else {
      message.error("开始考试失败");
    }
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
