import { Button, Form, Input, message, Modal, Pagination, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";

import {
  useBulkCreateStudents,
  useCreateStudent,
  useDeleteStudent,
  useStudentList,
  useStudentModal,
  useUpdateStudent,
} from "@/hooks/useStudents";
import { useTableHeight } from "@/hooks/useTableHeight";
import type { IStudentCreate, IStudentEditor, StudentListItem } from "@/types/main";

/**
 * 将 CSV 文本拆分成二维字符串数组，每行是一个子数组。
 * 手工处理引号、转义和换行以兼容常见的 CSV 格式。
 *
 * @param text - 完整的 CSV 文件内容。
 * @returns 按行分割的单元格数组。
 */
function parseCsvRows(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = "";
  let inQuotes = false;

  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];

    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          cell += '"';
          i += 1;
        } else {
          inQuotes = false;
        }
      } else {
        cell += ch;
      }
      continue;
    }

    if (ch === '"') {
      inQuotes = true;
      continue;
    }

    if (ch === ",") {
      row.push(cell);
      cell = "";
      continue;
    }

    if (ch === "\n") {
      row.push(cell);
      rows.push(row);
      row = [];
      cell = "";
      continue;
    }

    if (ch === "\r") {
      continue;
    }

    cell += ch;
  }

  if (cell.length > 0 || row.length > 0) {
    row.push(cell);
    rows.push(row);
  }

  return rows;
}

/**
 * 规范化表头字符串，去除前后空白、转换为小写并将空格替换为下划线。
 *
 * 用于将 CSV 头部与内部字段名称对齐。
 *
 * @param value - 原始表头文本。
 * @returns 处理后的规范化字符串。
 */
function normalizeHeader(value: string): string {
  return value.trim().toLowerCase().replace(/\s+/g, "_");
}

/**
 * 从 CSV 文本中提取学生数据。
 * 自动识别含或不含表头的格式，并根据多种可能的字段名称匹配学号和姓名。
 * 如果检测到重复学号会抛出错误。
 *
 * @param text - CSV 文件内容字符串。
 * @returns 解析得到的学生记录数组。
 * @throws 当 CSV 包含重复学号时。
 */
function parseStudentsFromCsv(text: string): IStudentCreate[] {
  const cleanText = text.replace(/^\uFEFF/, "");
  const rows = parseCsvRows(cleanText).filter((row) =>
    row.some((cell) => cell.trim().length > 0),
  );

  if (rows.length === 0) {
    return [];
  }

  const headers = rows[0].map((item) => normalizeHeader(item));
  const studentNoHeaderCandidates = ["student_no", "studentno", "学号"];
  const nameHeaderCandidates = ["name", "姓名", "student_name"];

  const studentNoIndex = headers.findIndex((h) =>
    studentNoHeaderCandidates.includes(h),
  );
  const nameIndex = headers.findIndex((h) => nameHeaderCandidates.includes(h));

  const hasHeader = studentNoIndex >= 0 && nameIndex >= 0;
  const startIndex = hasHeader ? 1 : 0;
  const resolvedStudentNoIndex = hasHeader ? studentNoIndex : 0;
  const resolvedNameIndex = hasHeader ? nameIndex : 1;

  if (resolvedStudentNoIndex < 0 || resolvedNameIndex < 0) {
    return [];
  }

  const parsed: IStudentCreate[] = [];
  const duplicateCheck = new Set<string>();

  for (let i = startIndex; i < rows.length; i += 1) {
    const current = rows[i];
    const studentNo = (current[resolvedStudentNoIndex] ?? "").trim();
    const name = (current[resolvedNameIndex] ?? "").trim();

    if (!studentNo || !name) {
      continue;
    }

    if (duplicateCheck.has(studentNo)) {
      throw new Error(`CSV 内存在重复学号: ${studentNo}`);
    }

    duplicateCheck.add(studentNo);
    parsed.push({ student_no: studentNo, name });
  }

  return parsed;
}

/**
 * 将任意错误对象转换为可显示的字符串。
 * 支持 Error 实例、字符串以及可 JSON 化的值。
 * 当无法序列化时返回默认消息。
 *
 * @param error - 捕获到的错误。
 * @returns 适合用于用户提示的错误描述。
 */
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

export function StudentsPage() {
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
  } = useStudentList();

  const { createStudent } = useCreateStudent();
  const { updateStudent } = useUpdateStudent();
  const { deleteStudent } = useDeleteStudent();
  const { bulkCreateStudents } = useBulkCreateStudents();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const paginationRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef, paginationRef);

  const studentModal = useStudentModal();
  const [importVisible, setImportVisible] = useState(false);
  const [importRows, setImportRows] = useState<IStudentCreate[]>([]);
  const [importing, setImporting] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [form] = Form.useForm();

  const importPreviewData = useMemo(
    () =>
      importRows.map((item, index) => ({
        key: `${item.student_no}-${index}`,
        student_no: item.student_no,
        name: item.name,
      })),
    [importRows],
  );

  useEffect(() => {
    if (!studentModal.visible) return;
    if (studentModal.formData) {
      form.setFieldsValue(studentModal.formData as any);
    } else {
      form.resetFields();
    }
  }, [studentModal.formData, studentModal.visible, form]);

  const handleDelete = (id: string) => {
    Modal.confirm({
      title: "确认删除",
      content: "删除后不可恢复，是否继续？",
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        const ok = await deleteStudent(id);
        if (ok) {
          message.success("删除成功");
          await refresh();
        } else {
          message.error("删除失败");
        }
      },
    });
  };

  const columns: ColumnsType<StudentListItem> = [
    {
      title: "学号",
      dataIndex: "student_no",
      key: "student_no",
      width: 240,
    },
    {
      title: "姓名",
      dataIndex: "name",
      key: "name",
      width: 220,
    },
    {
      title: "操作",
      dataIndex: "id",
      key: "id",
      width: 120,
      fixed: "right",
      render: (id: string, record) => (
        <div className="flex gap-2">
          <Button
            type="link"
            onClick={() =>
              studentModal.openEdit({
                id: record.id,
                student_no: record.student_no,
                name: record.name,
                created_at: record.created_at,
                updated_at: record.updated_at,
              })
            }
          >
            编辑
          </Button>
          <Button type="link" danger onClick={() => handleDelete(id)}>
            删除
          </Button>
        </div>
      ),
    },
  ];

  const onFinish = async (values: IStudentEditor) => {
    const payload: IStudentEditor = {
      id: values.id,
      student_no: values.student_no?.trim(),
      name: values.name?.trim(),
      created_at: values.created_at,
      updated_at: values.updated_at,
    };

    if (payload.id) {
      const ok = await updateStudent(payload);
      if (ok) {
        message.success("更新成功");
        studentModal.close();
        await refresh();
      } else {
        message.error("更新失败");
      }
      return;
    }

    const createPayload: IStudentCreate = {
      student_no: payload.student_no,
      name: payload.name,
      created_at: payload.created_at,
      updated_at: payload.updated_at,
    };

    const ok = await createStudent(createPayload);
    if (ok) {
      message.success("创建成功");
      studentModal.close();
      await refresh();
    } else {
      message.error("创建失败");
    }
  };

  const handleOpenImport = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) {
      return;
    }

    if (!file.name.toLowerCase().endsWith(".csv")) {
      message.error("请选择 CSV 文件");
      return;
    }

    try {
      const text = await file.text();
      const parsed = parseStudentsFromCsv(text);
      if (parsed.length === 0) {
        message.warning("未解析到可导入的学生数据，请检查 CSV 内容");
        return;
      }
      setImportRows(parsed);
      setImportVisible(true);
    } catch (error) {
      message.error(`解析 CSV 失败：${resolveErrorMessage(error)}`);
    }
  };

  const handleConfirmImport = async () => {
    if (importRows.length === 0) {
      message.warning("没有可导入的数据");
      return;
    }

    setImporting(true);
    try {
      const result = await bulkCreateStudents(importRows);
      if (!result) {
        message.error("批量导入失败");
        return;
      }

      message.success(`批量导入成功，共 ${result.length} 条`);
      setImportVisible(false);
      setImportRows([]);
      await refresh();
    } finally {
      setImporting(false);
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
          <div className="flex gap-4">
            <div className="flex-1 max-w-md">
              <Input
                value={inputKeyword}
                allowClear
                placeholder="按学号或姓名模糊查询"
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
            <Button type="primary" onClick={studentModal.openCreate}>
              新增学生
            </Button>
            <Button onClick={handleOpenImport}>批量导入</Button>
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv,text/csv"
              className="hidden"
              onChange={handleFileChange}
            />
          </div>
        </div>

        <Table<StudentListItem>
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
        title={studentModal.modalTitle}
        open={studentModal.visible}
        onCancel={studentModal.close}
        onOk={() => form.submit()}
        okText="确认"
        cancelText="取消"
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={onFinish}
          initialValues={studentModal.formData as any}
        >
          <Form.Item name="id" hidden>
            <Input />
          </Form.Item>

          <Form.Item name="created_at" hidden>
            <Input />
          </Form.Item>

          <Form.Item name="updated_at" hidden>
            <Input />
          </Form.Item>

          <Form.Item
            name="student_no"
            label="学号"
            rules={[{ required: true, message: "请输入学号" }]}
          >
            <Input />
          </Form.Item>

          <Form.Item
            name="name"
            label="姓名"
            rules={[{ required: true, message: "请输入姓名" }]}
          >
            <Input />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="批量导入学生"
        open={importVisible}
        onCancel={() => {
          setImportVisible(false);
          setImportRows([]);
        }}
        onOk={handleConfirmImport}
        okText="确认导入"
        cancelText="取消"
        confirmLoading={importing}
        width={760}
      >
        <p className="mb-3 text-gray-600">已解析 {importRows.length} 条学生记录</p>
        <Table
          rowKey="key"
          dataSource={importPreviewData}
          pagination={{ pageSize: 8, showSizeChanger: false }}
          size="small"
          columns={[
            {
              title: "学号",
              dataIndex: "student_no",
              key: "student_no",
            },
            {
              title: "姓名",
              dataIndex: "name",
              key: "name",
            },
          ]}
        />
      </Modal>
    </div>
  );
}
