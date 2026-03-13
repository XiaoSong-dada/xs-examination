import { useCallback, useEffect, useMemo, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { useExamStudents } from "@/hooks/useExamStudents";
import type { StudentListItem, MonitorTableItem } from "@/types/main";


/**
 * 实时监考页面 Hook：负责考试切换与学生监考表格数据。
 */
export function useMonitor() {
  const { exams, loading: examLoading } = useAllExamList();
  const {
    students,
    loading: studentLoading,
    fetchStudentsByExamId,
  } = useExamStudents();

  const [selectedExamId, setSelectedExamId] = useState<string>();

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    await fetchStudentsByExamId(selectedExamId);
  }, [fetchStudentsByExamId, selectedExamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const tableData = useMemo<MonitorTableItem[]>(
    () =>
      students.map((student: StudentListItem) => ({
        id: student.id,
        name: student.name,
        deviceIp: "-",
        linkStatus: "未连接",
        answerProgress: 0,
      })),
    [students],
  );

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading: studentLoading,
    refresh,
  } as const;
}
