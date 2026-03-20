use anyhow::Result;

use crate::network::device_network;
use crate::schemas::device_schema::DeviceRuntimeStatusDto;

pub struct DeviceService;

impl DeviceService {
    pub fn get_runtime_status() -> Result<DeviceRuntimeStatusDto> {
        let ip = device_network::resolve_device_ip()?;
        Ok(DeviceRuntimeStatusDto { ip })
    }
}
