import { useLayoutEffect, useState, type RefObject } from "react";

/**
 * 计算表格可用高度的 Hook。
 *
 * 它会观察容器、工具栏、分页等元素的尺寸变化，并返回一个建议的表格高度（像素）。
 *
 * @param containerRef 整个页面内容容器的 ref（用于计算可用高度）
 * @param toolbarRef 工具栏元素的 ref
 * @param paginationRef 分页容器的 ref
 * @param headerRef 可选的 header ref（如果应用有固定 header，可传入以扣除其高度）
 * @param minHeight 最小高度（像素），防止计算结果过小
 */
export function useTableHeight(
  containerRef: RefObject<HTMLElement | null>,
  toolbarRef?: RefObject<HTMLElement | null>,
  paginationRef?: RefObject<HTMLElement | null>,
  headerRef?: RefObject<HTMLElement | null>,
  minHeight = 120,
): number {
  const [height, setHeight] = useState<number>(400);

  useLayoutEffect(() => {
    if (!containerRef || !containerRef.current) {
      const fallback = Math.max(minHeight, window.innerHeight - 200);
      setHeight(fallback);
      return;
    }

    const update = () => {
      const container = containerRef.current as HTMLElement;
      const containerH = container.clientHeight || container.getBoundingClientRect().height || window.innerHeight;
      const toolbarH = toolbarRef?.current?.offsetHeight ?? 0;
      const paginationH = paginationRef?.current?.offsetHeight ?? 0;
      const headerH = headerRef?.current?.offsetHeight ?? 0;
      // 额外留白，避免与分页/边距贴得太紧
      const extra = 16;
      const avail = Math.floor(containerH - toolbarH - paginationH - headerH - extra);
      setHeight(Math.max(minHeight, avail));
    };

    update();

    const observers: ResizeObserver[] = [];
    try {
      const ro = new ResizeObserver(update);
      ro.observe(containerRef.current as Element);
      observers.push(ro);
      if (toolbarRef?.current) {
        const ro2 = new ResizeObserver(update);
        ro2.observe(toolbarRef.current as Element);
        observers.push(ro2);
      }
      if (paginationRef?.current) {
        const ro3 = new ResizeObserver(update);
        ro3.observe(paginationRef.current as Element);
        observers.push(ro3);
      }
    } catch (e) {
      // ResizeObserver 在极少数环境可能不存在，退回到 window resize
    }

    const onWin = () => update();
    window.addEventListener("resize", onWin);

    return () => {
      observers.forEach((o) => o.disconnect());
      window.removeEventListener("resize", onWin);
    };
  }, [containerRef, toolbarRef, paginationRef, headerRef, minHeight]);

  return height;
}
