import { dedupePaths } from "@/utils/pathUtils";
import type {
  IQuestionBankCreate,
  IQuestionBankEditor,
  QuestionBankItem,
  QuestionBankOption,
} from "@/types/main";

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

/**
 * 构造题库资源包导出的 xlsx 行数据。
 *
 * @param items - 勾选的题库题目列表。
 * @returns 返回可直接写入 xlsx 的行对象数组。
 */
export function toQuestionBankExportRows(items: QuestionBankItem[]): Array<Record<string, string | number>> {
  return items.map((item, index) => {
    const normalizedOptions = (item.options ?? []).map((option) => ({
      ...option,
      image_paths: (option.image_paths ?? []).map(toPackageRelativeImagePath),
    }));

    return {
      序号: index + 1,
      题型: item.type,
      题目内容: item.content,
      选项: JSON.stringify(normalizedOptions),
      答案: item.answer,
      分值: item.score,
      解析: item.explanation ?? "",
      题干图片: JSON.stringify(
        (item.content_image_paths ?? []).map(toPackageRelativeImagePath),
      ),
      选项图片映射: JSON.stringify(
        (item.options ?? []).reduce<Record<string, string[]>>((acc, option) => {
          if ((option.image_paths ?? []).length > 0) {
            acc[option.key] = option.image_paths.map(toPackageRelativeImagePath);
          }
          return acc;
        }, {}),
      ),
    };
  });
}

/**
 * 汇总题库导出所需的原始图片相对路径（用于后端打包）。
 *
 * @param items - 勾选的题库题目列表。
 * @returns 返回去重后的图片相对路径数组。
 */
export function collectQuestionBankExportImagePaths(items: QuestionBankItem[]): string[] {
  const allPaths: string[] = [];
  for (const item of items) {
    allPaths.push(...(item.content_image_paths ?? []));
    allPaths.push(
      ...(item.options ?? []).flatMap((option: QuestionBankOption) => option.image_paths ?? []),
    );
  }
  return dedupePaths(allPaths);
}

function toPackageRelativeImagePath(relativePath: string): string {
  const normalized = relativePath.trim().replace(/\\/g, "/");
  if (normalized.includes("/question-bank/content/")) {
    const name = normalized.split("/").pop() ?? normalized;
    return `assets/content/${name}`;
  }
  if (normalized.includes("/question-bank/options/")) {
    const name = normalized.split("/").pop() ?? normalized;
    return `assets/options/${name}`;
  }
  return normalized;
}