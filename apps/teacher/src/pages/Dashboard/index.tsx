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
import dayjs, { type Dayjs } from "@/utils/dayjs";
import type { ExamListItem, IExamCreate, IExamEditor } from "@/types/main";
import { useTableHeight } from "@/hooks/useTableHeight";
import {
  useExamModal,
  useExamList,
  useCreateExam,
  useUpdateExam,
  useDeleteExam,
} from "@/hooks/useExam";
import { getExamById } from "@/services/examService";
import { omitString } from "@/utils/utils";

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
  const { createExam } = useCreateExam();
  const { updateExam } = useUpdateExam();
  const { deleteExam } = useDeleteExam();

  const toPayload = (values: IExamEditor): IExamEditor => {
    const toTimestamp = (value?: Dayjs | number | null) => {
      if (value === undefined || value === null) {
        return null;
      }
      if (typeof value === "number") {
        return value;
      }
      return dayjs(value).valueOf();
    };

    return {
      id: values.id,
      title: values.title,
      description: values.description,
      start_time: toTimestamp(values.start_time),
      end_time: toTimestamp(values.end_time),
      pass_score: values.pass_score ?? 60,
      status: values.status ?? "draft",
      shuffle_questions: values.shuffle_questions ? 1 : 0,
      shuffle_options: values.shuffle_options ? 1 : 0,
    };
  };

  const handleEdit = async (id: string) => {
    try {
      const detail = await getExamById(id);
      examModal.openEdit({
        id: detail.id,
        title: detail.title,
        description: detail.description,
        start_time: detail.start_time ? dayjs(detail.start_time) : null,
        end_time: detail.end_time ? dayjs(detail.end_time) : null,
        pass_score: detail.pass_score,
        status: detail.status,
        shuffle_questions: detail.shuffle_questions === 1,
        shuffle_options: detail.shuffle_options === 1,
      });
    } catch (error) {
      console.error("获取考试详情失败", error);
      message.error("获取考试详情失败");
    }
  };

  const handleDelete = (id: string) => {
    Modal.confirm({
      title: "确认删除",
      content: "删除后不可恢复，是否继续？",
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        const ok = await deleteExam(id);
        if (ok) {
          message.success("删除成功");
          await refresh();
        } else {
          message.error("删除失败");
        }
      },
    });
  };

  const columns: ColumnsType<ExamListItem> = [
    {
      title: "考试标题",
      dataIndex: "title",
      key: "title",
      width: 200,
    },
    {
      title: "考试描述",
      dataIndex: "description",
      key: "description",
      width: 300,
      render: (description: string ) => (
        <span>{omitString(description ?? '', 20)}</span>
      ),
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
      title: "操作",
      dataIndex: "id",
      key: "id",
      width: 120,
      fixed: "right",
      render: (id: string) => (
        <div className="flex gap-2">
          <Button type="link" onClick={() => void handleEdit(id)}>
            编辑
          </Button>
          <Button type="link" danger onClick={() => handleDelete(id)}>
            删除
          </Button>
        </div>
      ),
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

  const onFinish = async (values: IExamEditor) => {
    const payload = toPayload(values);

    if (payload.id) {
      const ok = await updateExam(payload);
      if (ok) {
        message.success("更新成功");
        examModal.close();
        await refresh();
      } else {
        message.error("更新失败");
      }
      return;
    }

    const createPayload: IExamCreate = {
      title: payload.title,
      description: payload.description,
      start_time: payload.start_time,
      end_time: payload.end_time,
      pass_score: payload.pass_score,
      status: payload.status,
      shuffle_questions: payload.shuffle_questions,
      shuffle_options: payload.shuffle_options,
    };

    const ok = await createExam(createPayload);
    if (ok) {
      message.success("创建成功");
      examModal.close();
      await refresh();
    } else {
      message.error("创建失败");
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
          <Form.Item name="id" hidden>
            <Input />
          </Form.Item>

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
