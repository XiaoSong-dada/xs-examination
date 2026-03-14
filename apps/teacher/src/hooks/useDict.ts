import { useMemo } from "react";

/**
 * 提供项目内可复用的代码字典（数据字典）钩子集合。
 */
export function useExamState() {
  /**
   * 考试状态码映射：value -> label
   */
  const map = useMemo(
    () => ({
      draft: "草稿",
      published: "已发卷",
      active: "考试中",
      finished: "已结束",
      archived: "已归档",
    }),
    [],
  );

  return map;
}

export default useExamState;
