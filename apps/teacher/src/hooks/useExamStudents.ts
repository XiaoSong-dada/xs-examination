import { useCallback, useState } from "react";

import {
  getStudentListByExamId,
  importStudentsByExamId as importStudents,
} from "@/services/studentService";
import type { StudentListItem } from "@/types/main";

/**
 * 考试学生列表 Hook（不分页）。
 *
 * 按考试 ID 拉取并维护当前考试学生列表，供“学生引入”页面展示。
 */
export function useExamStudents() {
  const [students, setStudents] = useState<StudentListItem[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchStudentsByExamId = useCallback(async (examId?: string): Promise<StudentListItem[]> => {
    if (!examId) {
      setStudents([]);
      return [];
    }

    setLoading(true);
    try {
      const result = await getStudentListByExamId(examId);
      setStudents(result);
      return result;
    } catch (error) {
      console.error("[useExamStudents] 获取考试学生列表失败", error);
      setStudents([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const importStudentsByExamId = useCallback(
    async (examId: string, studentIds: string[]): Promise<StudentListItem[]> => {
      setLoading(true);
      try {
        const result = await importStudents(examId, studentIds);
        setStudents(result);
        return result;
      } catch (error) {
        console.error("[useExamStudents] 引入学生失败", error);
        throw error;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    students,
    loading,
    fetchStudentsByExamId,
    importStudentsByExamId,
    setStudents,
  } as const;
}
