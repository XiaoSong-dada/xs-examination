import { dedupePaths } from "@/utils/pathUtils";
import type { IQuestionBankCreate, IQuestionBankEditor } from "@/types/main";

export const questionTypeOptions = [
  { label: "单选题", value: "single" },
  { label: "多选题", value: "multiple" },
  { label: "判断题", value: "judge" },
  { label: "填空题", value: "blank" },
  { label: "论述题", value: "essay" },
];

export const optionTypeOptions = [
  { label: "纯文本", value: "text" },
  { label: "文字 + 图片", value: "text_with_image" },
];

/**
 * 将任意异常转换为可展示的提示文案。
 *
 * @param error - 捕获到的异常对象。
 * @returns 返回可直接展示给用户的错误信息。
 */
export function resolveQuestionBankErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return "未知错误";
  }
}

/**
 * 归一化题库编辑表单，避免空白与重复路径进入持久化层。
 *
 * @param values - 页面表单提交的题目编辑数据。
 * @returns 返回可用于新增/更新的规范化数据。
 */
export function normalizeQuestionBankPayload(
  values: IQuestionBankEditor,
): IQuestionBankEditor {
  return {
    ...values,
    type: values.type.trim(),
    content: values.content.trim(),
    content_image_paths: dedupePaths(values.content_image_paths ?? []),
    options: (values.options ?? []).map((item, index) => ({
      key: item.key.trim() || `${index + 1}`,
      text: item.text.trim(),
      option_type: item.option_type,
      image_paths: dedupePaths(item.image_paths ?? []),
    })),
    answer: values.answer.trim(),
    explanation: values.explanation?.trim() || undefined,
  };
}

/**
 * 将编辑态数据转换为新增接口所需的载荷结构。
 *
 * @param payload - 已归一化的编辑态题目数据。
 * @returns 返回新增题目接口参数。
 */
export function toQuestionBankCreatePayload(
  payload: IQuestionBankEditor,
): IQuestionBankCreate {
  return {
    type: payload.type,
    content: payload.content,
    content_image_paths: payload.content_image_paths,
    options: payload.options,
    answer: payload.answer,
    score: payload.score,
    explanation: payload.explanation,
    created_at: payload.created_at,
    updated_at: payload.updated_at,
  };
}