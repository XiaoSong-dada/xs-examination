import { Progress, Select, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";

import { useMonitor } from "@/hooks/useMonitor";
import { useTableHeight } from "@/hooks/useTableHeight";
import type {MonitorTableItem} from "@/types/main";

const linkStatusColorMap: Record<string, string> = {
  正常: "green",
  异常: "red",
  未连接: "gold",
  待分配: "default",
};

/**
 * 教师端实时监考页面。
 */
export function MonitorPage() {
  const {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading,
  } = useMonitor();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const columns: ColumnsType<MonitorTableItem> = [
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
      title: "连接状态",
      dataIndex: "linkStatus",
      key: "linkStatus",
      width: 160,
      render: (value: string) => (
        <Tag color={linkStatusColorMap[value] ?? "default"}>{value}</Tag>
      ),
    },
    {
      title: "答题进度",
      dataIndex: "answerProgress",
      key: "answerProgress",
      width: 240,
      render: (value: number) => <Progress percent={value} size="small" />,
    },
  ];

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
        </div>

        <Table<MonitorTableItem>
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
