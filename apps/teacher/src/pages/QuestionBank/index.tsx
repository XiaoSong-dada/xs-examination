import {
  Button,
  Form,
  Spin,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Space,
  Table,
  Image,
} from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";
import { formatTimestamp } from "@/utils/dayjs";

import {
  useCreateQuestionBankItem,
  useDeleteQuestionBankItem,
  useQuestionBankList,
  useQuestionBankModal,
  useUpdateQuestionBankItem,
} from "@/hooks/useQuestionBank";
import { useFileHooks } from "@/hooks/useFileHooks";
import { pickImageFilePaths } from "@/services/fileDialogService";
import { useTableHeight } from "@/hooks/useTableHeight";
import type {
  IQuestionBankCreate,
  IQuestionBankEditor,
  QuestionBankItem,
  QuestionBankOption,
} from "@/types/main";

const questionTypeOptions = [
  { label: "单选题", value: "single" },
  { label: "多选题", value: "multiple" },
  { label: "判断题", value: "judge" },
  { label: "填空题", value: "blank" },
  { label: "论述题", value: "essay" },
];

const optionTypeOptions = [
  { label: "纯文本", value: "text" },
  { label: "文字 + 图片", value: "text_with_image" },
];

function resolveErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return "未知错误";
  }
}

function dedupePaths(paths: string[]): string[] {
  return Array.from(new Set(paths.filter((item) => item.trim().length > 0)));
}

function normalizeQuestionBankPayload(
  values: IQuestionBankEditor,
): IQuestionBankEditor {
  return {
    ...values,
    type: values.type.trim(),
    content: values.content.trim(),
    content_image_paths: dedupePaths(values.content_image_paths ?? []),
    options: (values.options ?? []).map((item, index) => ({
      key: item.key.trim() || `${index + 1}`,
      text: item.text.trim(),
      option_type: item.option_type,
      image_paths: dedupePaths(item.image_paths ?? []),
    })),
    answer: values.answer.trim(),
    explanation: values.explanation?.trim() || undefined,
  };
}

/**
 * 教师端题目列表页面，提供独立题库的查询、新增、编辑与删除。
 *
 * @returns 返回题目列表表格与题目编辑弹窗。
 */
export function QuestionBankPage() {
  const {
    loading,
    inputKeyword,
    setInputKeyword,
    typeFilter,
    setTypeFilter,
    search,
    reset,
    total,
    dataSource,
    refresh,
  } = useQuestionBankList();
  const { createQuestionBankItem } = useCreateQuestionBankItem();
  const { updateQuestionBankItem } = useUpdateQuestionBankItem();
  const { deleteQuestionBankItem } = useDeleteQuestionBankItem();
  const { uploadQuestionBankImages, resolveImagePreviews } = useFileHooks();
  const questionModal = useQuestionBankModal();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const [form] = Form.useForm<IQuestionBankEditor>();
  const [contentPreviewMap, setContentPreviewMap] = useState<Record<string, string>>({});
  const [optionPreviewMap, setOptionPreviewMap] = useState<Record<string, string>>({});
  const [contentPreviewLoading, setContentPreviewLoading] = useState(false);
  const [optionPreviewLoading, setOptionPreviewLoading] = useState(false);
  const contentImagePaths =
    Form.useWatch("content_image_paths", { form, preserve: true }) ?? [];
  const watchedOptions =
    Form.useWatch("options", { form, preserve: true }) ?? [];
  const optionImagePaths = useMemo(
    () =>
      dedupePaths(
        (watchedOptions as QuestionBankOption[]).flatMap(
          (item) => item?.image_paths ?? [],
        ),
      ),
    [watchedOptions],
  );

  useEffect(() => {
    if (!questionModal.visible) {
      return;
    }

    if (questionModal.formData) {
      form.setFieldsValue({
        ...questionModal.formData,
        content_image_paths: questionModal.formData.content_image_paths ?? [],
        options: (questionModal.formData.options ?? []).map((item) => ({
          ...item,
          image_paths: item.image_paths ?? [],
        })),
      });
      return;
    }

    form.resetFields();
  }, [form, questionModal.formData, questionModal.visible]);

  useEffect(() => {
    if (!questionModal.visible || contentImagePaths.length === 0) {
      setContentPreviewMap({});
      setContentPreviewLoading(false);
      return;
    }

    let cancelled = false;
    setContentPreviewLoading(true);

    void (async () => {
      try {
        const previews = await resolveImagePreviews(contentImagePaths);
        if (!cancelled) {
          setContentPreviewMap(previews);
        }
      } catch (error) {
        if (!cancelled) {
          setContentPreviewMap({});
          message.warning(`题干图片预览加载失败：${resolveErrorMessage(error)}`);
        }
      } finally {
        if (!cancelled) {
          setContentPreviewLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [contentImagePaths, questionModal.visible, resolveImagePreviews]);

  useEffect(() => {
    if (!questionModal.visible || optionImagePaths.length === 0) {
      setOptionPreviewMap({});
      setOptionPreviewLoading(false);
      return;
    }

    let cancelled = false;
    setOptionPreviewLoading(true);

    void (async () => {
      try {
        const previews = await resolveImagePreviews(optionImagePaths);
        if (!cancelled) {
          setOptionPreviewMap(previews);
        }
      } catch (error) {
        if (!cancelled) {
          setOptionPreviewMap({});
          message.warning(`选项图片预览加载失败：${resolveErrorMessage(error)}`);
        }
      } finally {
        if (!cancelled) {
          setOptionPreviewLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [optionImagePaths, questionModal.visible, resolveImagePreviews]);

  const columns: ColumnsType<QuestionBankItem> = useMemo(
    () => [
      {
        title: "题型",
        dataIndex: "type",
        key: "type",
        width: 120,
        render: (value: string) =>
          questionTypeOptions.find((item) => item.value === value)?.label ??
          value,
      },
      {
        title: "题目内容",
        dataIndex: "content",
        key: "content",
        ellipsis: true,
      },
      {
        title: "答案",
        dataIndex: "answer",
        key: "answer",
        width: 120,
      },
      {
        title: "分值",
        dataIndex: "score",
        key: "score",
        width: 100,
      },
      {
        title: "更新时间",
        dataIndex: "updated_at",
        key: "updated_at",
        width: 180,
        render: (value: number) => formatTimestamp(value),
      },
      {
        title: "操作",
        key: "actions",
        width: 180,
        render: (_, record) => (
          <Space>
            <Button
              type="link"
              onClick={() =>
                questionModal.openEdit({
                  id: record.id,
                  type: record.type,
                  content: record.content,
                  content_image_paths: record.content_image_paths,
                  options: record.options,
                  answer: record.answer,
                  score: record.score,
                  explanation: record.explanation,
                  created_at: record.created_at,
                  updated_at: record.updated_at,
                })
              }
            >
              编辑
            </Button>
            <Button
              danger
              type="link"
              onClick={() => {
                Modal.confirm({
                  title: "确认删除题目",
                  content: "删除后不可恢复，是否继续？",
                  okText: "删除",
                  cancelText: "取消",
                  okButtonProps: { danger: true },
                  onOk: async () => {
                    try {
                      await deleteQuestionBankItem(record.id);
                      message.success("删除成功");
                      await refresh();
                    } catch (error) {
                      message.error(resolveErrorMessage(error));
                    }
                  },
                });
              }}
            >
              删除
            </Button>
          </Space>
        ),
      },
    ],
    [deleteQuestionBankItem, questionModal, refresh],
  );

  const handlePickContentImages = async () => {
    const selected = await pickImageFilePaths(true);
    if (selected.length === 0) {
      return;
    }

    const uploaded = await uploadQuestionBankImages(selected, "content");
    const paths = uploaded.map((item) => item.relative_path);
    form.setFieldValue(
      "content_image_paths",
      dedupePaths([
        ...(form.getFieldValue("content_image_paths") ?? []),
        ...paths,
      ]),
    );
  };

  const handlePickOptionImages = async (index: number) => {
    const selected = await pickImageFilePaths(true);
    if (selected.length === 0) {
      return;
    }

    const uploaded = await uploadQuestionBankImages(selected, "options");
    const paths = uploaded.map((item) => item.relative_path);
    const current = (form.getFieldValue(["options", index, "image_paths"]) ??
      []) as string[];
    form.setFieldValue(
      ["options", index, "image_paths"],
      dedupePaths([...current, ...paths]),
    );
  };

  const removeContentImage = (path: string) => {
    form.setFieldValue(
      "content_image_paths",
      contentImagePaths.filter((item: string) => item !== path),
    );
  };

  const removeOptionImage = (index: number, path: string) => {
    const current = (
      (watchedOptions[index]?.image_paths ?? []) as string[]
    ).filter((item) => item !== path);
    form.setFieldValue(["options", index, "image_paths"], current);
  };

  const handleSubmit = async (values: IQuestionBankEditor) => {
    const payload = normalizeQuestionBankPayload(values);

    try {
      if (payload.id) {
        await updateQuestionBankItem(payload);
        message.success("更新成功");
      } else {
        const createPayload: IQuestionBankCreate = {
          type: payload.type,
          content: payload.content,
          content_image_paths: payload.content_image_paths,
          options: payload.options,
          answer: payload.answer,
          score: payload.score,
          explanation: payload.explanation,
          created_at: payload.created_at,
          updated_at: payload.updated_at,
        };
        await createQuestionBankItem(createPayload);
        message.success("新增成功");
      }

      questionModal.close();
      form.resetFields();
      await refresh();
    } catch (error) {
      message.error(resolveErrorMessage(error));
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
          <div className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-64 max-w-xl">
              <Input
                value={inputKeyword}
                allowClear
                placeholder="按题目内容、答案或解析搜索"
                onChange={(event) => setInputKeyword(event.target.value)}
                onPressEnter={search}
              />
            </div>
            <Select
              className="w-44"
              allowClear
              placeholder="题型筛选"
              value={typeFilter}
              options={questionTypeOptions}
              onChange={(value) => setTypeFilter(value)}
            />
            <Space>
              <Button type="primary" onClick={search}>
                搜索
              </Button>
              <Button onClick={reset}>重置</Button>
            </Space>
          </div>
          <div className="flex items-center justify-between gap-4">
            <Space>
              <Button type="primary" onClick={questionModal.openCreate}>
                新增题目
              </Button>
              <Button onClick={() => void refresh()}>刷新</Button>
            </Space>
            <div className="text-sm text-slate-500">共 {total} 条题目</div>
          </div>
        </div>

        <Table<QuestionBankItem>
          rowKey="id"
          loading={loading}
          dataSource={dataSource}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>

      <Modal
        title={questionModal.modalTitle}
        width={1080}
        open={questionModal.visible}
        onCancel={questionModal.close}
        onOk={() => form.submit()}
        okText="确认"
        cancelText="取消"
        destroyOnHidden
      >
        <Form<IQuestionBankEditor>
          form={form}
          layout="vertical"
          initialValues={questionModal.formData ?? undefined}
          onFinish={(values) => void handleSubmit(values)}
        >
          <Form.Item name="id" hidden>
            <Input />
          </Form.Item>

          <div className="grid grid-cols-2 gap-4">
            <Form.Item
              name="type"
              label="题型"
              rules={[{ required: true, message: "请选择题型" }]}
            >
              <Select options={questionTypeOptions} />
            </Form.Item>

            <Form.Item
              name="score"
              label="分值"
              rules={[{ required: true, message: "请输入分值" }]}
            >
              <InputNumber min={0} className="w-full" />
            </Form.Item>
          </div>

          <Form.Item
            name="content"
            label="题目内容"
            rules={[{ required: true, message: "请输入题目内容" }]}
          >
            <Input.TextArea rows={4} placeholder="请输入题干文本" />
          </Form.Item>

          <Form.Item label="题干图片">
            <div className="space-y-3">
              <Space>
                <Button onClick={() => void handlePickContentImages()}>
                  选择图片
                </Button>
                <Button
                  onClick={() => form.setFieldValue("content_image_paths", [])}
                >
                  清空
                </Button>
              </Space>
              <div className="flex flex-wrap gap-2">
                {contentImagePaths.length === 0 ? (
                  <span className="text-sm text-slate-400">未选择题干图片</span>
                ) : (
                  <Image.PreviewGroup>
                    {contentImagePaths.map((path: string) => (
                      <div key={path} className="relative">
                        {contentPreviewMap[path] ? (
                          <Image
                            src={contentPreviewMap[path]}
                            alt={path}
                            style={{
                              maxWidth: "120px",
                              maxHeight: "120px",
                              objectFit: "cover",
                            }}
                          />
                        ) : (
                          <div className="w-[120px] h-[120px] border border-slate-200 rounded flex items-center justify-center text-xs text-slate-500 px-2 text-center">
                            {contentPreviewLoading ? <Spin size="small" /> : "预览不可用"}
                          </div>
                        )}
                        <Button
                          size="small"
                          shape="circle"
                          danger
                          className="absolute -top-2 -right-2"
                          onClick={() => removeContentImage(path)}
                        >
                          ×
                        </Button>
                      </div>
                    ))}
                  </Image.PreviewGroup>
                )}
              </div>
            </div>
          </Form.Item>

          <Form.List name="options">
            {(fields, { add, remove }) => (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="text-sm font-medium text-slate-700">
                    选项设置
                  </div>
                  <Button
                    onClick={() =>
                      add({
                        key: `${fields.length + 1}`,
                        text: "",
                        option_type: "text",
                        image_paths: [],
                      })
                    }
                    type="primary"
                  >
                    添加选项
                  </Button>
                </div>

                <div className="h-[300px] overflow-y-auto pr-1 space-y-4">
                  {fields.map((field, index) => {
                    const { key: _fieldReactKey, ...fieldProps } = field;
                    const option = (watchedOptions[index] ?? {
                      key: "",
                      text: "",
                      option_type: "text",
                      image_paths: [],
                    }) as QuestionBankOption;
                    const optionImagePaths = option.image_paths ?? [];

                    return (
                      <div
                        key={field.key}
                        className="rounded-lg border border-slate-200 p-4 space-y-3"
                      >
                        <div className="grid grid-cols-[100px_1fr_180px_96px] gap-3 items-start">
                          <Form.Item
                            {...fieldProps}
                            name={[field.name, "key"]}
                            label="选项键"
                            rules={[
                              { required: true, message: "请输入选项键" },
                            ]}
                          >
                            <Input placeholder="如 A" />
                          </Form.Item>
                          <Form.Item
                            {...fieldProps}
                            name={[field.name, "text"]}
                            label="选项文本"
                          >
                            <Input placeholder="请输入选项文本" />
                          </Form.Item>
                          <Form.Item
                            {...fieldProps}
                            name={[field.name, "option_type"]}
                            label="选项类型"
                            rules={[
                              { required: true, message: "请选择选项类型" },
                            ]}
                          >
                            <Select options={optionTypeOptions} />
                          </Form.Item>
                          <div className="pt-[30px] text-right">
                            <Button danger onClick={() => remove(field.name)}>
                              删除
                            </Button>
                          </div>
                        </div>

                        <div className="space-y-3">
                          <Space>
                            <Button
                              onClick={() => void handlePickOptionImages(index)}
                            >
                              选择附件图片
                            </Button>
                            <Button
                              onClick={() =>
                                form.setFieldValue(
                                  ["options", index, "image_paths"],
                                  [],
                                )
                              }
                            >
                              清空图片
                            </Button>
                          </Space>
                          <div className="flex flex-wrap gap-2">
                            {optionImagePaths.length === 0 ? (
                              <span className="text-sm text-slate-400">
                                未选择附件图片
                              </span>
                            ) : (
                              optionImagePaths.map((path) => (
                                <div
                                  key={`${field.key}-${path}`}
                                  className="border border-slate-200 rounded p-2 space-y-2"
                                >
                                  <div className="relative">
                                    {optionPreviewMap[path] ? (
                                      <Image
                                        src={optionPreviewMap[path]}
                                        alt={path}
                                        width={88}
                                        height={88}
                                        style={{ objectFit: "cover" }}
                                      />
                                    ) : (
                                      <div className="w-[88px] h-[88px] border border-slate-200 rounded flex items-center justify-center text-xs text-slate-500">
                                        {optionPreviewLoading ? <Spin size="small" /> : "预览不可用"}
                                      </div>
                                    )}

                                    <Button
                                      size="small"
                                      shape="circle"
                                      danger
                                      className="absolute -top-1 -right-1"
                                      onClick={() => removeOptionImage(index, path)}
                                    >
                                      ×
                                    </Button>
                                  </div>
                                </div>
                              ))
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </Form.List>

          <div className="grid grid-cols-2 gap-4 mt-4">
            <Form.Item
              name="answer"
              label="答案"
              rules={[{ required: true, message: "请输入答案" }]}
            >
              <Input placeholder="如 A 或 A,B" />
            </Form.Item>
            <Form.Item name="explanation" label="解析">
              <Input.TextArea rows={3} placeholder="可选，填写题目解析" />
            </Form.Item>
          </div>
        </Form>
      </Modal>
    </div>
  );
}
