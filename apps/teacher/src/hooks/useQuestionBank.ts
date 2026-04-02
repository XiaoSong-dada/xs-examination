import { useCallback, useEffect, useMemo, useState } from "react";

import {
  createQuestionBankItem as createQuestionBankItemService,
  deleteQuestionBankItem as deleteQuestionBankItemService,
  getQuestionBankItems,
  updateQuestionBankItem as updateQuestionBankItemService,
} from "@/services/questionService";
import type {
  IQuestionBankCreate,
  IQuestionBankEditor,
  QuestionBankItem,
  QuestionBankOption,
  UseQuestionBankListResult,
} from "@/types/main";
import { deepClone } from "@/utils/utils";

function createDefaultQuestionBankOptions(): QuestionBankOption[] {
  return ["A", "B", "C", "D"].map((key) => ({
    key,
    text: "",
    option_type: "text",
    image_paths: [],
  }));
}

const defaultQuestionBankEditor: IQuestionBankEditor = {
  id: "",
  type: "single",
  content: "",
  content_image_paths: [],
  options: createDefaultQuestionBankOptions(),
  answer: "",
  score: 0,
  explanation: "",
  created_at: undefined,
  updated_at: undefined,
};

/**
 * 管理教师端独立题库列表的拉取、筛选与刷新。
 *
 * @returns 返回题库列表状态、搜索条件和刷新能力。
 */
export function useQuestionBankList(): UseQuestionBankListResult {
  const [loading, setLoading] = useState(false);
  const [allItems, setAllItems] = useState<QuestionBankItem[]>([]);
  const [inputKeyword, setInputKeyword] = useState("");
  const [appliedKeyword, setAppliedKeyword] = useState("");
  const [typeFilter, setTypeFilter] = useState<string>();
  const [appliedTypeFilter, setAppliedTypeFilter] = useState<string>();

  /**
   * 拉取全局题库题目并更新本地状态。
   *
   * @returns 请求完成后无返回值。
   */
  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    try {
      const result = await getQuestionBankItems();
      setAllItems(result);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  /**
   * 提交当前搜索条件并应用到列表筛选。
   *
   * @returns 无返回值。
   */
  const search = useCallback(() => {
    setAppliedKeyword(inputKeyword);
    setAppliedTypeFilter(typeFilter);
  }, [inputKeyword, typeFilter]);

  /**
   * 清空当前关键字和题型筛选条件。
   *
   * @returns 无返回值。
   */
  const reset = useCallback(() => {
    setInputKeyword("");
    setAppliedKeyword("");
    setTypeFilter(undefined);
    setAppliedTypeFilter(undefined);
  }, []);

  const dataSource = useMemo(() => {
    const normalizedKeyword = appliedKeyword.trim().toLowerCase();
    return allItems.filter((item) => {
      const matchesKeyword =
        !normalizedKeyword ||
        item.content.toLowerCase().includes(normalizedKeyword) ||
        item.answer.toLowerCase().includes(normalizedKeyword) ||
        item.explanation?.toLowerCase().includes(normalizedKeyword);

      const matchesType = !appliedTypeFilter || item.type === appliedTypeFilter;
      return Boolean(matchesKeyword && matchesType);
    });
  }, [allItems, appliedKeyword, appliedTypeFilter]);

  return {
    loading,
    inputKeyword,
    appliedKeyword,
    setInputKeyword,
    typeFilter,
    appliedTypeFilter,
    setTypeFilter,
    search,
    reset,
    total: dataSource.length,
    dataSource,
    refresh,
  };
}

/**
 * 提供新增全局题库题目的异步方法。
 *
 * @returns 返回新增题目方法。
 */
export function useCreateQuestionBankItem() {
  const createQuestionBankItem = useCallback(async (data: IQuestionBankCreate) => {
    return createQuestionBankItemService(data);
  }, []);

  return { createQuestionBankItem };
}

/**
 * 提供更新全局题库题目的异步方法。
 *
 * @returns 返回更新题目方法。
 */
export function useUpdateQuestionBankItem() {
  const updateQuestionBankItem = useCallback(async (data: IQuestionBankEditor) => {
    return updateQuestionBankItemService(data);
  }, []);

  return { updateQuestionBankItem };
}

/**
 * 提供删除全局题库题目的异步方法。
 *
 * @returns 返回删除题目方法。
 */
export function useDeleteQuestionBankItem() {
  const deleteQuestionBankItem = useCallback(async (id: string) => {
    return deleteQuestionBankItemService(id);
  }, []);

  return { deleteQuestionBankItem };
}

/**
 * 管理题库新增/编辑弹窗的标题、显隐与表单默认值。
 *
 * @returns 返回弹窗状态和打开/关闭能力。
 */
export function useQuestionBankModal() {
  const [visible, setVisible] = useState(false);
  const [modalTitle, setModalTitle] = useState("新增题目");
  const [formData, setFormData] = useState<IQuestionBankEditor | null>(null);

  const openCreate = useCallback(() => {
    setModalTitle("新增题目");
    setFormData(deepClone(defaultQuestionBankEditor));
    setVisible(true);
  }, []);

  const openEdit = useCallback((data: IQuestionBankEditor) => {
    setModalTitle("编辑题目");
    setFormData({
      ...deepClone(data),
      options: data.options.length > 0 ? deepClone(data.options) : createDefaultQuestionBankOptions(),
    });
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
  } as const;
}