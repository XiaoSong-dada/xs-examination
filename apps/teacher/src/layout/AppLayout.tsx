import {
	AppstoreOutlined,
	BarChartOutlined,
	EditOutlined,
	ImportOutlined,
	MonitorOutlined,
	PlusSquareOutlined,
} from "@ant-design/icons";
import { Layout, Menu } from "antd";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

const { Content, Sider } = Layout;

const menuItems = [
	{ key: "/", icon: <AppstoreOutlined />, label: "考试列表" },
	{ key: "/exam/create", icon: <PlusSquareOutlined />, label: "新建考试" },
	{
		key: "/question/import",
		icon: <ImportOutlined />,
		label: "题库导入",
	},
	{ key: "/monitor", icon: <MonitorOutlined />, label: "实时监考" },
	{ key: "/grading", icon: <EditOutlined />, label: "主观阅卷" },
	{ key: "/report", icon: <BarChartOutlined />, label: "成绩报告" },
];

/**
 * 教师端主布局，渲染左侧导航与右侧内容区。
 *
 * @returns 返回包含菜单和页面内容插槽的应用骨架。
 */
export function AppLayout() {
	const navigate = useNavigate();
	const { pathname } = useLocation();

	return (
		<Layout className="h-screen">
			<Sider theme="light" className="border-r border-gray-200">
				<div className="h-16 border-b border-gray-100 text-primary flex items-center justify-center text-lg font-semibold">
					XS 考试系统
				</div>
				<Menu
					mode="inline"
					selectedKeys={[pathname]}
					items={menuItems}
					onClick={({ key }) => navigate(key)}
					className="h-[calc(100%-64px)] border-none"
				/>
			</Sider>
			<Layout>
				<Content className="bg-gray-50 p-6 overflow-auto">
					<Outlet />
				</Content>
			</Layout>
		</Layout>
	);
}
