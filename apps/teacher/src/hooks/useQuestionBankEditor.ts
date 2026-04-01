import { Form, message } from "antd";
import { useEffect, useMemo, useState } from "react";

import { useFileHooks } from "@/hooks/useFileHooks";
import {
  useCreateQuestionBankItem,
  useQuestionBankModal,
  useUpdateQuestionBankItem,
} from "@/hooks/useQuestionBank";
import { pickImageFilePaths } from "@/services/fileDialogService";
import {
  normalizeQuestionBankPayload,
  resolveQuestionBankErrorMessage,
  toQuestionBankCreatePayload,
} from "@/services/questionBankEditorService";
import { dedupePaths } from "@/utils/pathUtils";
import type { IQuestionBankEditor, QuestionBankOption } from "@/types/main";

/**
 * 管理题库编辑弹窗的表单状态、图片预览与新增/更新提交。
 *
 * @param refresh - 题库列表刷新方法。
 * @returns 返回弹窗状态、表单实例、预览状态和交互处理函数。
 */
export function useQuestionBankEditor(refresh: () => Promise<void>) {
  const { createQuestionBankItem } = useCreateQuestionBankItem();
  const { updateQuestionBankItem } = useUpdateQuestionBankItem();
  const { uploadQuestionBankImages, resolveImagePreviews } = useFileHooks();
  const questionModal = useQuestionBankModal();

  const [form] = Form.useForm<IQuestionBankEditor>();
  const [contentPreviewMap, setContentPreviewMap] = useState<
    Record<string, string>
  >({});
  const [optionPreviewMap, setOptionPreviewMap] = useState<
    Record<string, string>
  >({});
  const [contentPreviewLoading, setContentPreviewLoading] = useState(false);
  const [optionPreviewLoading, setOptionPreviewLoading] = useState(false);

  const contentImagePaths =
    Form.useWatch("content_image_paths", { form, preserve: true }) ?? [];
  const watchedOptions = Form.useWatch("options", { form, preserve: true }) ?? [];

  const optionImagePaths = useMemo(
    () =>
      dedupePaths(
        (watchedOptions as QuestionBankOption[]).flatMap(
          (item) => item?.image_paths ?? [],
        ),
      ),
    [watchedOptions],
  );

  useEffect(() => {
    if (!questionModal.visible) {
      return;
    }

    if (questionModal.formData) {
      form.setFieldsValue({
        ...questionModal.formData,
        content_image_paths: questionModal.formData.content_image_paths ?? [],
        options: (questionModal.formData.options ?? []).map((item) => ({
          ...item,
          image_paths: item.image_paths ?? [],
        })),
      });
      return;
    }

    form.resetFields();
  }, [form, questionModal.formData, questionModal.visible]);

  useEffect(() => {
    if (!questionModal.visible || contentImagePaths.length === 0) {
      setContentPreviewMap((prev) => (Object.keys(prev).length === 0 ? prev : {}));
      setContentPreviewLoading((prev) => (prev === false ? prev : false));
      return;
    }

    let cancelled = false;
    setContentPreviewLoading(true);

    void (async () => {
      try {
        const previews = await resolveImagePreviews(contentImagePaths);
        if (!cancelled) {
          setContentPreviewMap(previews);
        }
      } catch (error) {
        if (!cancelled) {
          setContentPreviewMap({});
          message.warning(
            `题干图片预览加载失败：${resolveQuestionBankErrorMessage(error)}`,
          );
        }
      } finally {
        if (!cancelled) {
          setContentPreviewLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [contentImagePaths, questionModal.visible, resolveImagePreviews]);

  useEffect(() => {
    if (!questionModal.visible || optionImagePaths.length === 0) {
      setOptionPreviewMap((prev) => (Object.keys(prev).length === 0 ? prev : {}));
      setOptionPreviewLoading((prev) => (prev === false ? prev : false));
      return;
    }

    let cancelled = false;
    setOptionPreviewLoading(true);

    void (async () => {
      try {
        const previews = await resolveImagePreviews(optionImagePaths);
        if (!cancelled) {
          setOptionPreviewMap(previews);
        }
      } catch (error) {
        if (!cancelled) {
          setOptionPreviewMap({});
          message.warning(
            `选项图片预览加载失败：${resolveQuestionBankErrorMessage(error)}`,
          );
        }
      } finally {
        if (!cancelled) {
          setOptionPreviewLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [optionImagePaths, questionModal.visible, resolveImagePreviews]);

  const handlePickContentImages = async (): Promise<void> => {
    const selected = await pickImageFilePaths(true);
    if (selected.length === 0) {
      return;
    }

    const uploaded = await uploadQuestionBankImages(selected, "content");
    const paths = uploaded.map((item) => item.relative_path);
    form.setFieldValue(
      "content_image_paths",
      dedupePaths([...(form.getFieldValue("content_image_paths") ?? []), ...paths]),
    );
  };

  const handlePickOptionImages = async (index: number): Promise<void> => {
    const selected = await pickImageFilePaths(true);
    if (selected.length === 0) {
      return;
    }

    const uploaded = await uploadQuestionBankImages(selected, "options");
    const paths = uploaded.map((item) => item.relative_path);
    const current = (form.getFieldValue(["options", index, "image_paths"]) ??
      []) as string[];
    form.setFieldValue(["options", index, "image_paths"], dedupePaths([...current, ...paths]));
  };

  const removeContentImage = (path: string): void => {
    form.setFieldValue(
      "content_image_paths",
      contentImagePaths.filter((item: string) => item !== path),
    );
  };

  const removeOptionImage = (index: number, path: string): void => {
    const current = ((watchedOptions[index]?.image_paths ?? []) as string[]).filter(
      (item) => item !== path,
    );
    form.setFieldValue(["options", index, "image_paths"], current);
  };

  const handleSubmit = async (values: IQuestionBankEditor): Promise<void> => {
    const payload = normalizeQuestionBankPayload(values);

    try {
      if (payload.id) {
        await updateQuestionBankItem(payload);
        message.success("更新成功");
      } else {
        await createQuestionBankItem(toQuestionBankCreatePayload(payload));
        message.success("新增成功");
      }

      questionModal.close();
      form.resetFields();
      await refresh();
    } catch (error) {
      message.error(resolveQuestionBankErrorMessage(error));
    }
  };

  return {
    form,
    questionModal,
    contentImagePaths,
    watchedOptions: watchedOptions as QuestionBankOption[],
    contentPreviewMap,
    optionPreviewMap,
    contentPreviewLoading,
    optionPreviewLoading,
    handlePickContentImages,
    handlePickOptionImages,
    removeContentImage,
    removeOptionImage,
    handleSubmit,
  };
}