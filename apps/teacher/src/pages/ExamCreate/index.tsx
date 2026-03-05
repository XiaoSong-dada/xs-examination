import { Button, Form, Input, DatePicker, InputNumber, Select, Switch, message, Space } from "antd";
import type { Dayjs } from "@/utils/dayjs";
import type { IExamCreate } from "@/types/main";
import { createExam } from "@/services/examService";
import useExam from "@/hooks/useExam";

const { TextArea } = Input;

/**
 * 新建考试页面表单。
 *
 * 表单字段与数据库 `exams` 表字段对应：
 * - title, description, start_time, end_time, pass_score, status, shuffle_questions, shuffle_options
 *
 * @returns 新建考试表单组件
 */
export function ExamCreatePage() {
  const [form] = Form.useForm();
  const { getStatusOptions } = useExam();

  const onFinish = async (values: IExamCreate) => {
    const payload = {
      title: values.title,
      description: values.description,
      start_time: values.start_time ? (values.start_time as Dayjs).valueOf() : null,
      end_time: values.end_time ? (values.end_time as Dayjs).valueOf() : null,
      pass_score: values.pass_score ?? 60,
      status: values.status,
      shuffle_questions: values.shuffle_questions ? 1 : 0,
      shuffle_options: values.shuffle_options ? 1 : 0,
    };

    try {
      await createExam(payload as any);
      message.success("创建成功");
      form.resetFields();
    } catch (err) {
      console.error(err);
      message.error("创建失败，请重试");
    }
  };

  const onReset = () => form.resetFields();

  return (
    <div className="p-4 w-full h-full overflow-auto">
      <Form
        form={form}
        layout="vertical"
        className="bg-white rounded-lg p-6 shadow-sm h-full overflow-auto"
        onFinish={onFinish}
        initialValues={{ status: "draft", pass_score: 60 }}
      >
        <Form.Item label="考试标题" name="title" rules={[{ required: true, message: "请输入考试标题" }] }>
          <Input placeholder="例如：期中考试 - 2026" />
        </Form.Item>

        <Form.Item label="考试须知" name="description">
          <TextArea rows={4} placeholder="可填写考试须知或说明，支持简单富文本" />
        </Form.Item>

        <div className="grid grid-cols-2 gap-4">
          <Form.Item label="开始时间" name="start_time">
            <DatePicker showTime className="w-full" />
          </Form.Item>

          <Form.Item label="结束时间" name="end_time">
            <DatePicker showTime className="w-full" />
          </Form.Item>
        </div>

        <div className="grid grid-cols-4 gap-4">
          <Form.Item label="及格分" name="pass_score">
            <InputNumber min={0} max={100} className="w-full" />
          </Form.Item>

          <Form.Item label="状态" name="status">
            <Select options={getStatusOptions()} />
          </Form.Item>

          <Form.Item label="随机题序" name="shuffle_questions" valuePropName="checked">
            <Switch />
          </Form.Item>

          <Form.Item label="随机选项" name="shuffle_options" valuePropName="checked">
            <Switch />
          </Form.Item>
        </div>

        <Form.Item>
          <Space>
            <Button type="primary" htmlType="submit">
              新增
            </Button>
            <Button onClick={onReset}>清空</Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
