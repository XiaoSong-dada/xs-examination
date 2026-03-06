import {
  Button,
  Input,
  Pagination,
  Table,
  Tag,
  Modal,
  Form,
  Select,
  Switch,
  DatePicker,
  InputNumber,
  message,
} from "antd";
import { useRef, useEffect } from "react";
import type { ColumnsType } from "antd/es/table";
import type { ExamListItem, IExamCreate, IExamEditor } from "@/types/main";
import { useTableHeight } from "@/hooks/useTableHeight";
import { useExamModal, useExamList, useCreateExam } from "@/hooks/useExam";

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
    refresh,
  } = useExamList();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const paginationRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef, paginationRef);
  const {createExam} = useCreateExam();

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

  const examModal = useExamModal();

  const [form] = Form.useForm();

  useEffect(() => {
    if (!examModal.visible) return; 

    if (examModal.formData) {
      form.setFieldsValue(examModal.formData as any);
    } else {
      form.resetFields();
    }
  }, [examModal.formData, form]);

  const handleCreateExam = () => {
    examModal.openCreate();
  };

  const onFinish = (values: IExamEditor) => {
    console.log("保存考试：", values);

    if(values.id){

    }
    else {
      createExam(values as IExamCreate).then(res =>{
        if(res){
          examModal.close();
          message.success("创建成功!");
          refresh();
        }
      });
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
          className="bg-white rounded-lg flex flex-col gap-5 pb-4 w-full"
        >
          <div className="flex gap-4 ">
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
          <div className="flex gap-2">
            <Button type="primary" onClick={handleCreateExam}>
              新建考试
            </Button>
            <Button>批量导入</Button>
          </div>
        </div>

        <Table<ExamListItem>
          rowKey="id"
          loading={loading}
          dataSource={dataSource}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />

        <div
          ref={paginationRef}
          className="mt-4 flex justify-end overflow-x-auto"
        >
          <Pagination
            className="xs-pagination-nowrap"
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
      <Modal
        title={examModal.modalTitle}
        open={examModal.visible}
        onCancel={examModal.close}
        onOk={() => form.submit()}
        okText="确认"
        cancelText="取消"
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={onFinish}
          initialValues={examModal.formData as any}
        >
          <Form.Item
            name="title"
            label="考试标题"
            rules={[{ required: true, message: "请输入考试标题" }]}
          >
            <Input />
          </Form.Item>

          <Form.Item name="description" label="描述">
            <Input.TextArea rows={3} />
          </Form.Item>

          <Form.Item name="status" label="状态">
            <Select options={examModal.statusOptions} />
          </Form.Item>
          <div className="flex gap-4 justify-between">
            <Form.Item label="开始时间" name="start_time">
              <DatePicker showTime style={{ width: "100%" }} />
            </Form.Item>

            <Form.Item label="结束时间" name="end_time">
              <DatePicker showTime style={{ width: "100%" }} />
            </Form.Item>
          </div>

          <Form.Item name="pass_score" label="及格分数">
            <InputNumber min={0} style={{ width: "100%" }} />
          </Form.Item>

          <div className="flex gap-4">
            <Form.Item
              name="shuffle_questions"
              label="乱序题目"
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>
            <Form.Item
              name="shuffle_options"
              label="乱序选项"
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>
          </div>
        </Form>
      </Modal>
    </div>
  );
}
