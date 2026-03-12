import {
  AppstoreOutlined,
  BarChartOutlined,
  ControlOutlined,
  DesktopOutlined,
  EditOutlined,
  ImportOutlined,
  MonitorOutlined,
  TeamOutlined,
} from "@ant-design/icons";
import { Layout, Menu } from "antd";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

const { Content, Sider, Header } = Layout;

const menuItems = [
  { key: "/", icon: <AppstoreOutlined />, label: "考试列表" },
  { key: "/devices", icon: <DesktopOutlined />, label: "设备列表" },

  { key: "/students", icon: <TeamOutlined />, label: "学生列表" },
  {
    key: "/question/import",
    icon: <ImportOutlined />,
    label: "题库导入",
  },
  {
    key: "/students/import",
    icon: <ImportOutlined />,
    label: "学生引入",
  },
  { key: "/devices/assign", icon: <DesktopOutlined />, label: "分配设备" },
  { key: "/exam/manage", icon: <ControlOutlined />, label: "考试管理" },
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
        <Header className="bg-white border-b border-gray-200 p-4">
          <div className="text-xl font-medium">
            {menuItems.find((item) => item.key === pathname)?.label ?? "页面"}
          </div>
        </Header>
        <Content className="bg-gray-50 p-6 overflow-auto">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}
