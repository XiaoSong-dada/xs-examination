import { RouterProvider } from "react-router-dom";
import { router } from "./router";
import { message } from "antd";
/**
 * 应用入口组件，负责挂载全局路由系统。
 *
 * @returns 返回 RouterProvider 根节点。
 */
export default function App() {
  const [ _messageApi,contextHolder ]=message.useMessage();

  return (
  <>
    {contextHolder}
    <RouterProvider router={router} />
  </>
)
}
