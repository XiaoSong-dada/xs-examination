import type { ReactNode } from "react";
import AppHeader from "@/layout/AppHeader";

interface Props {
  children?: ReactNode;
}

export default function AppLayout({ children }: Props) {
  return (
    <div className="min-h-screen bg-slate-50 text-slate-900">
      <AppHeader />
      <section className="mx-auto w-full h-full max-w-7xl flex-1 px-4 py-4">
        {/* 内容区：答题区（children） */}
        {children}
      </section>
    </div>
  );
}
