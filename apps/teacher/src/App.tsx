import { RouterProvider } from "react-router-dom";
import { router } from "./router";

/**
 * 应用入口组件，负责挂载全局路由系统。
 *
 * @returns 返回 RouterProvider 根节点。
 */
export default function App() {
  return <RouterProvider router={router} />;
}
