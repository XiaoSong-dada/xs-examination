import { Button, Modal, Select, Table, Transfer, message } from "antd";
import type { TransferProps } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { useExamStudents } from "@/hooks/useExamStudents";
import { useStudentList } from "@/hooks/useStudents";
import { useTableHeight } from "@/hooks/useTableHeight";
import type { StudentListItem } from "@/types/main";

/**
 * 教师端考试学生引入页面。
 *
 * @returns 返回考试筛选与当前考试学生列表。
 */
export function StudentImportPage() {
  const { exams, loading: examLoading } = useAllExamList();
  const {
    students,
    loading: studentLoading,
    fetchStudentsByExamId,
    importStudentsByExamId,
  } = useExamStudents();
  const {
    dataSource: allStudents,
    loading: allStudentLoading,
    refresh: refreshAllStudents,
    setPageSize,
  } = useStudentList();

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [importModalVisible, setImportModalVisible] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [targetKeys, setTargetKeys] = useState<string[]>([]);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  useEffect(() => {
    const fetchStudents = async () => {
      await fetchStudentsByExamId(selectedExamId);
    };

    void fetchStudents();
  }, [fetchStudentsByExamId, selectedExamId]);

  useEffect(() => {
    setPageSize(Number.MAX_SAFE_INTEGER);
  }, [setPageSize]);

  useEffect(() => {
    if (importModalVisible) {
      setTargetKeys(students.map((item) => item.id));
    }
  }, [importModalVisible, students]);

  const handleImportStudents = async () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    try {
      await refreshAllStudents();
      setTargetKeys(students.map((item) => item.id));
      setImportModalVisible(true);
    } catch {
      message.error("获取学生数据失败");
    }
  };

  const handleTransferChange: TransferProps["onChange"] = (nextTargetKeys) => {
    setTargetKeys(nextTargetKeys as string[]);
  };

  const handleConfirmImport = async () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    setSubmitting(true);
    try {
      await importStudentsByExamId(selectedExamId, targetKeys);
      await fetchStudentsByExamId(selectedExamId);
      setImportModalVisible(false);
      message.success(`成功引入 ${targetKeys.length} 名学生`);
    } catch (error) {
      console.error("引入学生失败", error);
      message.error("引入学生失败，请稍后重试");
    } finally {
      setSubmitting(false);
    }
  };

  const transferDataSource = useMemo(
    () =>
      allStudents.map((item) => ({
        key: item.id,
        title: item.name,
        description: item.student_no,
      })),
    [allStudents],
  );

  const columns: ColumnsType<StudentListItem> = [
    {
      title: "学号",
      dataIndex: "student_no",
      key: "student_no",
      width: 220,
    },
    {
      title: "姓名",
      dataIndex: "name",
      key: "name",
      width: 180,
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
          className="bg-white rounded-lg flex flex-col gap-5 pb-4 w-full"
        >
          <div className="flex gap-2">
            <Select
              className="w-full max-w-md"
              placeholder="请选择考试"
              value={selectedExamId}
              loading={examLoading}
              options={examOptions}
              onChange={setSelectedExamId}
            />
            <Button type="primary" onClick={handleImportStudents}>
              引入学生
            </Button>
          </div>
        </div>


        <Table<StudentListItem>
          rowKey="id"
          loading={studentLoading}
          dataSource={students}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>

      <Modal
        title="引入学生"
        width={900}
        open={importModalVisible}
        onCancel={() => setImportModalVisible(false)}
        onOk={() => void handleConfirmImport()}
        okText="确认引入"
        cancelText="取消"
        confirmLoading={submitting}
      >
        <Transfer
          dataSource={transferDataSource}
          targetKeys={targetKeys}
          onChange={handleTransferChange}
          render={(item) => `${item.title}（${item.description}）`}
          styles={{ section: { width: 400, height: 420 } }}
          actions={["引入", "移除"]}
          titles={["全部学生", "当前考试已引入学生"]}
          showSearch
          disabled={allStudentLoading || submitting}
        />
      </Modal>
    </div>
  );
}
