type ImageListProps = {
  images: string[];
};

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
          <img
            src={image}
            alt={`题目图片 ${index + 1}`}
            className="h-40 w-full object-cover"
          />
        </li>
      ))}
    </ul>
  );
}