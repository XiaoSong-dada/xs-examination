import type { ReactNode } from "react";
import AppHeader from "@/layout/AppHeader";

interface Props {
  children?: ReactNode;
}

export default function AppLayout({ children }: Props) {
  return (
    <div className="text-slate-900 h-full flex flex-col">
      <AppHeader />
      <section className="mx-auto w-full h-full">
        {/* 内容区：答题区（children） */}
        {children}
      </section>
    </div>
  );
}
