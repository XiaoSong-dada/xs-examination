import { createBrowserRouter } from "react-router-dom";
import { AppLayout } from "../layout/AppLayout";
import { DashboardPage } from "../pages/Dashboard/index";
import { ExamCreatePage } from "../pages/ExamCreate/index";
import { GradingPage } from "../pages/Grading/index";
import { MonitorPage } from "../pages/Monitor/index";
import { QuestionImportPage } from "../pages/QuestionImport/index";
import { ReportPage } from "../pages/Report/index";

/**
 * 教师端路由配置，定义骨架布局与各业务页面路径。
 *
 * @returns 返回可供 RouterProvider 使用的浏览器路由实例。
 */
export const router = createBrowserRouter([
	{
		path: "/",
		element: <AppLayout />,
		children: [
			{
				index: true,
				element: <DashboardPage />,
			},
			{
				path: "exam/create",
				element: <ExamCreatePage />,
			},
			{
				path: "question/import",
				element: <QuestionImportPage />,
			},
			{
				path: "monitor",
				element: <MonitorPage />,
			},
			{
				path: "grading",
				element: <GradingPage />,
			},
			{
				path: "report",
				element: <ReportPage />,
			},
		],
	},
]);
