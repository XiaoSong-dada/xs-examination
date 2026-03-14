type AnswerListProps = {
  options: string[];
  selectedOption: number | null;
  onSelect: (index: number) => void;
};

const optionLabels = ["A", "B", "C", "D", "E", "F"];

export default function AnswerList({
  options,
  selectedOption,
  onSelect,
}: AnswerListProps) {
  return (
    <div className="space-y-3">
      {options.map((option, index) => {
        const isSelected = selectedOption === index;
        return (
          <button
            key={`${option}-${index}`}
            type="button"
            onClick={() => onSelect(index)}
            className={`flex w-full items-start gap-3 rounded-md border px-4 py-3 text-left transition ${
              isSelected
                ? "border-sky-500 bg-sky-50 text-sky-700"
                : "border-slate-200 bg-white text-slate-700 hover:border-sky-300"
            }`}
          >
            <span className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-current text-xs font-semibold">
              {optionLabels[index] ?? `${index + 1}`}
            </span>
            <span className="text-sm leading-6">{option}</span>
          </button>
        );
      })}
    </div>
  );
}