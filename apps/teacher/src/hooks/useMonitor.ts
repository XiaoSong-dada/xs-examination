import { useCallback, useEffect, useMemo, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { getStudentDeviceConnectionStatusByExamId } from "@/services/studentService";
import type { MonitorTableItem, StudentDeviceConnectionStatusItem } from "@/types/main";


/**
 * 实时监考页面 Hook：负责考试切换与学生监考表格数据。
 */
export function useMonitor() {
  const { exams, loading: examLoading } = useAllExamList();

  const [students, setStudents] = useState<StudentDeviceConnectionStatusItem[]>([]);
  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [tableLoading, setTableLoading] = useState(false);

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    if (!selectedExamId) {
      setStudents([]);
      return;
    }

    setTableLoading(true);
    try {
      const list = await getStudentDeviceConnectionStatusByExamId(selectedExamId);
      setStudents(list);
    } finally {
      setTableLoading(false);
    }
  }, [selectedExamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!selectedExamId) {
      return;
    }

    const timer = window.setInterval(() => {
      void refresh();
    }, 5000);

    return () => {
      window.clearInterval(timer);
    };
  }, [refresh, selectedExamId]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const tableData = useMemo<MonitorTableItem[]>(
    () =>
      students.map((student: StudentDeviceConnectionStatusItem) => ({
        id: student.student_id,
        name: student.student_name,
        deviceIp: student.ip_addr ?? "-",
        linkStatus: student.connection_status,
        answerProgress: student.progress_percent ?? 0,
      })),
    [students],
  );

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading,
    refresh,
  } as const;
}
