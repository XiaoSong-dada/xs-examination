import { Button, message, Select, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";
import { useDeviceAssign, type DeviceAssignRow } from "@/hooks/useDeviceAssign";
import { useTableHeight } from "@/hooks/useTableHeight";

export function DeviceAssignPage() {
  const {
    loading,
    assigning,
    examOptions,
    selectedExamId,
    setSelectedExamId,
    tableData,
    randomAssign,
    clearAssign,
    studentCount,
    deviceCount,
    assignedCount,
  } = useDeviceAssign();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const columns: ColumnsType<DeviceAssignRow> = [
    {
      title: "设备 IP",
      dataIndex: "ip",
      key: "ip",
      width: 240,
    },
    {
      title: "设备名称",
      dataIndex: "name",
      key: "name",
      width: 240,
    },
    {
      title: "分配考生",
      key: "student",
      width: 260,
      render: (_, record) => {
        if (!record.assigned) {
          return <Tag>未分配</Tag>;
        }

        return (
          <span>
            {record.student_name}（{record.student_no}）
          </span>
        );
      },
    },
    {
      title: "状态",
      key: "assigned",
      width: 120,
      render: (_, record) =>
        record.assigned ? <Tag color="green">已分配</Tag> : <Tag color="default">待分配</Tag>,
    },
  ];

  const handleRandomAssign = () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    const ok = randomAssign();
    if (!ok) {
      message.warning("当前考试暂无考生或设备数据为空");
      return;
    }

    message.success("随机分配完成");
  };

  return (
    <div className="space-y-4 h-full">
      <div
        ref={containerRef}
        className="bg-white rounded-lg border border-gray-200 p-4 h-full"
      >
        <div
          ref={toolbarRef}
          className="bg-white rounded-lg flex flex-col gap-5 pb-4 w-full"
        >
          <div className="flex gap-4 items-center">
            <div className="w-[360px]">
              <Select
                value={selectedExamId}
                placeholder="请选择考试"
                options={examOptions}
                onChange={setSelectedExamId}
                allowClear
                className="w-full"
              />
            </div>
            <div className="flex items-center gap-2">
              <Button
                type="primary"
                loading={assigning}
                onClick={handleRandomAssign}
              >
                随机分配考生
              </Button>
              <Button onClick={clearAssign}>清空分配</Button>
            </div>
          </div>
          <div className="text-sm text-gray-500">
            设备数：{deviceCount}，考生数：{studentCount}，已分配：{assignedCount}
          </div>
        </div>

        <Table<DeviceAssignRow>
          rowKey="id"
          loading={loading}
          dataSource={tableData}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>
    </div>
  );
}
