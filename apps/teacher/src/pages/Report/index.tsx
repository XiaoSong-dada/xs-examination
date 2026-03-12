import { Button, Progress, Select, Table, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";

import { useReport, type ReportTableItem } from "@/hooks/useReport";
import { useTableHeight } from "@/hooks/useTableHeight";

/**
 * 教师端成绩报告页面。
 */
export function ReportPage() {
  const {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading,
    exporting,
    exportReport,
  } = useReport();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const columns: ColumnsType<ReportTableItem> = [
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
      title: "答题进度",
      dataIndex: "answerProgress",
      key: "answerProgress",
      width: 220,
      render: (value: number) => <Progress percent={value} size="small" />,
    },
    {
      title: "分值",
      dataIndex: "score",
      key: "score",
      width: 160,
    },
  ];

  const handleExport = () => {
    const ok = exportReport();
    if (ok) {
      message.success("成绩报告导出成功");
    } else {
      message.error("成绩报告导出失败");
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
          <Button type="primary" loading={exporting} onClick={handleExport}>
            导出成绩
          </Button>
        </div>

        <Table<ReportTableItem>
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
