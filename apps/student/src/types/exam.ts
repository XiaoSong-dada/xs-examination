export interface ExamQuestionOption {
  key: string;
  text: string;
  optionType?: "text" | "text_with_image";
  imagePaths?: string[];
}
