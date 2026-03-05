import { useCallback } from "react";
import useExamState from "./useDict";

/**
 * useExam 钩子：提供与考试相关的通用工具/辅助函数
 */
export function useExam() {
  const examState = useExamState();

  /**
   * 返回用于 Ant Design `Select` 的 `options` 数组
   */
  const getStatusOptions = useCallback(() => {
    return Object.entries(examState).map(([value, label]) => ({
      label,
      value,
    }));
  }, [examState]);

  return {
    getStatusOptions,
    examState,
  };
}

export default useExam;
