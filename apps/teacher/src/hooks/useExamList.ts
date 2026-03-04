import { useCallback, useEffect, useMemo, useState } from "react";
import { getExamList, type ExamListItem } from "../services/examService";

export interface UseExamListResult {
  loading: boolean;
  inputKeyword: string;
  appliedKeyword: string;
  setInputKeyword: (value: string) => void;
  search: () => void;
  reset: () => void;
  page: number;
  pageSize: number;
  setPage: (value: number) => void;
  setPageSize: (value: number) => void;
  total: number;
  dataSource: ExamListItem[];
  refresh: () => Promise<void>;
}

/**
 * 考试列表页面数据 Hook，负责加载数据、模糊查询与分页。
 *
 * @returns 返回页面渲染所需的状态与操作方法。
 */
export function useExamList(): UseExamListResult {
  const [loading, setLoading] = useState(false);
  const [inputKeyword, setInputKeyword] = useState("");
  const [appliedKeyword, setAppliedKeyword] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const [allExams, setAllExams] = useState<ExamListItem[]>([]);

  /**
   * 拉取考试列表并更新本地状态。
   *
   * @returns 无返回值；失败时打印错误并保持已有数据。
   */
  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    try {
      const result = await getExamList();
      setAllExams(result);
    } catch (error) {
      console.error("[useExamList] 获取考试列表失败", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  /**
   * 执行标题搜索并重置到第一页。
   *
   * @returns 无返回值。
   */
  const search = useCallback(() => {
    setAppliedKeyword(inputKeyword);
    setPage(1);
  }, [inputKeyword]);

  /**
   * 重置搜索条件并重置到第一页。
   *
   * @returns 无返回值。
   */
  const reset = useCallback(() => {
    setInputKeyword("");
    setAppliedKeyword("");
    setPage(1);
  }, []);

  const filteredExams = useMemo(() => {
    const normalizedKeyword = appliedKeyword.trim().toLowerCase();
    if (!normalizedKeyword) {
      return allExams;
    }

    return allExams.filter((item) =>
      item.title.toLowerCase().includes(normalizedKeyword),
    );
  }, [allExams, appliedKeyword]);

  const total = filteredExams.length;

  const dataSource = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredExams.slice(start, start + pageSize);
  }, [filteredExams, page, pageSize]);

  useEffect(() => {
    const maxPage = Math.max(1, Math.ceil(total / pageSize));
    if (page > maxPage) {
      setPage(maxPage);
    }
  }, [page, pageSize, total]);

  return {
    loading,
    inputKeyword,
    appliedKeyword,
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
  };
}
