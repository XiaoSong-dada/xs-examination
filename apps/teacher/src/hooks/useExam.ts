import { useCallback, useState, useEffect, useMemo } from "react";
import useExamState from "./useDict";
import { IExamCreate } from "@/types/main";
import { createExam as create, getExamList } from "@/services/examService";
import { deepClone } from "@/utils/utils";
import type { UseExamListResult, ExamListItem, IExamEditor } from "@/types/main";


/**
 * 考试列表页面数据 Hook，负责加载数据、模糊查询与分页。
 *
 * @returns 返回页面渲染所需的状态与操作方法。
 */
/**
 * Hook used by the exam list page. 负责加载考试数据、
 * 处理模糊搜索和分页逻辑。
 *
 * 返回对象包含：
 * - loading：是否正在请求
 * - inputKeyword/appliedKeyword：搜索关键字状态
 * - setInputKeyword/search/reset：搜索相关操作
 * - page/pageSize/setPage/setPageSize：分页状态
 * - total/dataSource：当前分页结果
 * - refresh：手动刷新列表
 */
export function useExamList(): UseExamListResult {
  const [loading, setLoading] = useState<boolean>(false);
  const [allExams, setAllExams] = useState<ExamListItem[]>([]);
  const [inputKeyword, setInputKeyword] = useState("");
  const [appliedKeyword, setAppliedKeyword] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);


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

/**
 * useExam 钩子：提供与考试相关的通用工具/辅助函数
 */
/**
 * 通用考试相关工具 Hook 。
 *
 * 提供状态字典映射以及其他通用 helper。
 * 目前只有 getStatusOptions 方法，用于生成 Select 组件的 options。
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
/**
 * 提供创建考试的异步函数。
 *
 * 调用后会向后端提交数据并返回 boolean 表示是否成功。
 */
export const useCreateExam = () => {
  const createExam = useCallback(async (data: IExamCreate ) => {
    try {
      await create(data);
      return true;
    }
    catch (e) {
      console.error("创建考试失败", e);
      return false;
    }

  }, [])


  return { createExam }
}

const default_create_exam_data: IExamEditor = {
  id:'',
  title: "",
  description: "",
  start_time: null,
  end_time: null,
  pass_score: 60,
  status: "draft",
  shuffle_questions: 0,
  shuffle_options: 0,
}


/**
 * useExamModal：管理新增/编辑考试对话框的状态和表单数据
 */
/**
 * 管理新增/编辑考试对话框状态的 Hook。
 *
 * 返回值包含：
 * - modalTitle/visible：弹窗标题和可见性
 * - formData/setFormData：表单的当前值
 * - openCreate/openEdit/close：控制对话框打开/关闭
 * - statusOptions：状态下拉数据
 */
export function useExamModal() {
  const { getStatusOptions } = useExam();

  const [visible, setVisible] = useState(false);
  const [modalTitle, setModalTitle] = useState("新建考试");
  const [formData, setFormData] = useState<IExamEditor | null>(null);

  const openCreate = useCallback(() => {
    setModalTitle("新建考试");
    setFormData(deepClone(default_create_exam_data));
    setVisible(true);
  }, []);

  const openEdit = useCallback((data: IExamEditor) => {
    setModalTitle("编辑考试");
    setFormData(data);
    setVisible(true);
  }, []);

  const close = useCallback(() => {
    setVisible(false);
  }, []);

  return {
    modalTitle,
    visible,
    formData,
    setFormData,
    openCreate,
    openEdit,
    close,
    statusOptions: getStatusOptions(),
  } as const;
}


export default useExam;
