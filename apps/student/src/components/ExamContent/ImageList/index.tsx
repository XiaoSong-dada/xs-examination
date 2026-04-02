import { Image } from "antd";

type ImageListProps = {
  images: string[];
};

/**
 * 渲染题干图片列表，支持点击预览、缩放与旋转。
 * @param props 图片列表参数。
 * @returns 返回题干图片网格或空状态提示。
 */
export default function ImageList({ images }: ImageListProps) {
  if (images.length === 0) {
    return (
      <div className="rounded-md border border-dashed border-slate-300 bg-slate-50 p-6 text-center text-sm text-slate-500">
        暂无题目图片
      </div>
    );
  }

  return (
    <ul className="grid grid-cols-1 gap-3 sm:grid-cols-2">
      {images.map((image, index) => (
        <li
          key={`${image}-${index}`}
          className="overflow-hidden rounded-md border border-slate-200 bg-white"
        >
          <Image
            src={image}
            alt={`题目图片 ${index + 1}`}
            width="100%"
            height={160}
            style={{ objectFit: "cover" }}
          />
        </li>
      ))}
    </ul>
  );
}