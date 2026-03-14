import { useCallback, useEffect, useMemo, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import type { MonitorTableItem, StudentDeviceAssignItem } from "@/types/main";
import { useDeviceAssign } from "./useDeviceAssign";


/**
 * 实时监考页面 Hook：负责考试切换与学生监考表格数据。
 */
export function useMonitor() {
  const { exams, loading: examLoading } = useAllExamList();
  const {
    getAssignStudentByExamId,
    loading: studentLoading,
  } = useDeviceAssign();

  const [students, setStudents] = useState<StudentDeviceAssignItem[]>([]);
  const [selectedExamId, setSelectedExamId] = useState<string>();

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    const students = await getAssignStudentByExamId(selectedExamId ?? "");
    setStudents(students ?? []);
  }, [getAssignStudentByExamId, selectedExamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const tableData = useMemo<MonitorTableItem[]>(
    () =>
      students.map((student: StudentDeviceAssignItem) => ({
        id: student.student_id,
        name: student.student_name,
        deviceIp: student.ip_addr ?? "-",
        linkStatus: student.ip_addr ? "已连接" : "未连接",
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
