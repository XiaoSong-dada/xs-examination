import { Button, Input, Pagination, Table, Tag } from "antd";
import { useRef } from "react";
import type { ColumnsType } from "antd/es/table";
import { useExamList } from "../../hooks/useExamList";
import type { ExamListItem } from "../../services/examService";

const statusColorMap: Record<string, string> = {
  draft: "default",
  published: "blue",
  active: "green",
  paused: "orange",
  finished: "purple",
};

/**
 * 教师端考试列表首页视图组件。
 *
 * @returns 返回包含 Toolbar、列表表格和分页器的页面。
 */
export function DashboardPage() {
  const {
    loading,
    inputKeyword,
    setInputKeyword,
    search,
    reset,
    page,
    pageSize,
    setPage,
    setPageSize,
    total,
    dataSource,
  } = useExamList();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const paginationRef = useRef<HTMLDivElement | null>(null);

  const columns: ColumnsType<ExamListItem> = [
    {
      title: "考试标题",
      dataIndex: "title",
      key: "title",
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
      width: 140,
      render: (status: string) => (
        <Tag color={statusColorMap[status] ?? "default"}>{status}</Tag>
      ),
    },
    {
      title: "考试 ID",
      dataIndex: "id",
      key: "id",
    },
  ];

  return (
    <div className="space-y-4 h-full">
      <div ref={containerRef} className="bg-white rounded-lg border border-gray-200 p-4 h-full">
        <div ref={toolbarRef} className="bg-white rounded-lg flex items-center justify-between gap-4 pb-4">
          <div className="flex-1 max-w-md">
            <Input
              value={inputKeyword}
              allowClear
              placeholder="按考试标题模糊查询"
              onChange={(event) => setInputKeyword(event.target.value)}
              onPressEnter={search}
            />
          </div>
          <div className="flex items-center gap-2">
            <Button type="primary" onClick={search}>
              搜索
            </Button>
            <Button onClick={reset}>重置</Button>
          </div>
        </div>

        <Table<ExamListItem>
          rowKey="id"
          loading={loading}
          dataSource={dataSource}
          columns={columns}
          pagination={false}
        />

        <div ref={paginationRef} className="mt-4 flex justify-end">
          <Pagination
            current={page}
            pageSize={pageSize}
            total={total}
            showSizeChanger
            showQuickJumper
            showTotal={(value) => `共 ${value} 条`}
            onChange={(nextPage, nextPageSize) => {
              setPage(nextPage);
              if (nextPageSize !== pageSize) {
                setPageSize(nextPageSize);
                setPage(1);
              }
            }}
          />
        </div>
      </div>
    </div>
  );
}
