import { useCallback, useState } from "react";

import {
  bulkImportQuestions,
  getQuestionListByExamId,
  importQuestionPackageByExamId as importQuestionPackageByExamIdService,
} from "@/services/questionService";
import type { Question } from "@/types/main";

/**
 * 题库数据管理 Hook。
 *
 * - 维护当前页面使用的题目数组
 * - 提供按考试 ID 获取题库的方法（不分页）
 */
export const useQuestion = () => {
  const [questions, setQuestions] = useState<Question[]>([]);
  const [loading, setLoading] = useState(false);

  /**
   * 按考试 ID 拉取题库（不分页）并写入本地 questions。
   *
   * @param examId - 考试 ID
   * @returns 拉取成功后返回题目数组
   */
  const fetchQuestionsByExamId = useCallback(async (examId?: string): Promise<Question[]> => {
    if (!examId) {
      setQuestions([]);
      return [];
    }

    setLoading(true);
    try {
      const result = await getQuestionListByExamId({ exam_id: examId });
      setQuestions(result);
      return result;
    } catch (error) {
      console.error("[useQuestion] 获取题目列表失败", error);
      setQuestions([]);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 批量导入题目（按 exam_id 覆盖导入）：先删后插。
   *
   * @param examId - 当前选中的考试 ID
   * @param importQuestions - 待导入题目数组
   * @returns 导入完成后的题目数组
   */
  const importQuestionsByExamId = useCallback(
    async (examId: string, importQuestions: Question[]): Promise<Question[]> => {
      setLoading(true);
      try {
        const result = await bulkImportQuestions({
          exam_id: examId,
          questions: importQuestions,
        });
        setQuestions(result);
        return result;
      } catch (error) {
        console.error("[useQuestion] 批量导入题目失败", error);
        throw error;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  /**
   * 按考试导入题目资源包（zip），并覆盖写入题目。
   *
   * @param examId - 当前选中的考试 ID。
   * @param packagePath - 资源包绝对路径。
   * @returns 导入完成后的题目数组。
   */
  const importQuestionPackageByExamId = useCallback(
    async (examId: string, packagePath: string): Promise<Question[]> => {
      setLoading(true);
      try {
        const result = await importQuestionPackageByExamIdService({
          exam_id: examId,
          package_path: packagePath,
        });
        setQuestions(result);
        return result;
      } catch (error) {
        console.error("[useQuestion] 导入题目资源包失败", error);
        throw error;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    questions,
    setQuestions,
    loading,
    fetchQuestionsByExamId,
    importQuestionsByExamId,
    importQuestionPackageByExamId,
  } as const;
};
