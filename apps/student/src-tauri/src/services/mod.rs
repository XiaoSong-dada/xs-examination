//! services 层：封装业务逻辑（配置管理、连接管理等），调用 adapters 提供的数据访问能力。
//!
//! 当前为占位模块，后续会实现 `TeacherConfigService` 等。

pub mod teacher_endpoints_service;
pub mod exam_runtime_service;
pub mod device_service;
pub mod ws_reconnect_service;
pub mod file_asset_service;
