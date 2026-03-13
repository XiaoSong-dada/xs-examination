import { Button, message, Select, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useRef } from "react";
import { useDeviceAssign } from "@/hooks/useDeviceAssign";
import { useTableHeight } from "@/hooks/useTableHeight";
import type { DeviceAssignRow } from "@/types/main";

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
      title: "考生",
      key: "student",
      width: 240,
      render: (_, record) => (
        <span>
          {record.student_name}（{record.student_no}）
        </span>
      ),
    },
    {
      title: "设备 IP",
      dataIndex: "ip_addr",
      key: "ip_addr",
      width: 260,
      render: (value?: string) => value ?? <Tag>未分配</Tag>,
    },
    {
      title: "设备名称",
      dataIndex: "device_name",
      key: "device_name",
      width: 240,
      render: (value?: string) => value ?? <Tag>未分配</Tag>,
    },
    {
      title: "状态",
      key: "assigned",
      width: 120,
      render: (_, record) =>
        record.assigned ? (
          <Tag color="green">已分配</Tag>
        ) : (
          <Tag color="default">待分配</Tag>
        ),
    },
  ];

  const handleRandomAssign = async () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    const ok = await randomAssign();
    if (!ok) {
      message.warning("当前考试暂无考生或设备数据为空");
      return;
    }

    message.success("随机分配完成");
  };

  const handleClearAssign = async () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    await clearAssign();
    message.success("已清空分配");
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
              <Button loading={assigning} onClick={handleClearAssign}>
                清空分配
              </Button>
            </div>
          </div>
          <div className="text-sm text-gray-500">
            设备数：{deviceCount}，考生数：{studentCount}，已分配：
            {assignedCount}
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
