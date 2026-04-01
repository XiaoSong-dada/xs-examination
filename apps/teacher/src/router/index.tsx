import { createBrowserRouter } from "react-router-dom";
import { AppLayout } from "../layout/AppLayout";
import { DashboardPage } from "../pages/Dashboard/index";
import { DeviceAssignPage } from "../pages/DeviceAssign/index";
import { DevicesPage } from "../pages/Devices/index";
import { ExamManagePage } from "../pages/ExamManage/index";
import { GradingPage } from "../pages/Grading/index";
import { MonitorPage } from "../pages/Monitor/index";
import { QuestionBankPage } from "../pages/QuestionBank/index";
import { QuestionImportPage } from "../pages/QuestionImport/index";
import { ReportPage } from "../pages/Report/index";
import { StudentImportPage } from "../pages/StudentImport/index";
import { StudentsPage } from "../pages/Students/index";

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
				path: "devices",
				element: <DevicesPage />,
			},
			{
				path: "students/assign",
				element: <DeviceAssignPage />,
			},
			{
				path: "exam/manage",
				element: <ExamManagePage />,
			},
			{
				path: "students",
				element: <StudentsPage />,
			},
			{
				path: "questions",
				element: <QuestionBankPage />,
			},
			{
				path: "question/import",
				element: <QuestionImportPage />,
			},
			{
				path: "students/import",
				element: <StudentImportPage />,
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
