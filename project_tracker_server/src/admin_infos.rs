use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, path::PathBuf};
use systemstat::{saturating_sub_bytes, Platform, System};

use crate::{get_logs_as_string, ConnectedClient, CpuUsageAverage};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminInfos {
	pub connected_native_gui_clients: Vec<SocketAddr>,
	pub connected_web_clients: Vec<SocketAddr>,
	pub cpu_usage: f32,
	pub cpu_temp: Option<f32>,
	pub ram_info: String,
	pub uptime: String,
	pub latest_logs_of_the_day: String,
}

impl AdminInfos {
	pub fn generate(
		connected_clients: HashSet<ConnectedClient>,
		cpu_usage_avg: &CpuUsageAverage,
		log_filepath: &PathBuf,
	) -> Self {
		let mut connected_native_gui_clients = Vec::new();
		let mut connected_web_clients = Vec::new();

		for connected_client in connected_clients {
			match connected_client {
				ConnectedClient::NativeGUI(addr) => connected_native_gui_clients.push(addr),
				ConnectedClient::Web(addr) => connected_web_clients.push(addr),
			}
		}

		let sys = System::new();

		let cpu_temp = sys.cpu_temp().ok();

		let ram_info = match sys.memory() {
			Ok(mem) => format!(
				"{} / {}",
				saturating_sub_bytes(mem.total, mem.free),
				mem.total
			),
			Err(_) => "failed to get ram info".to_string(),
		};

		let uptime = match sys.uptime() {
			Ok(uptime) => format_duration(uptime).to_string(),
			Err(_) => "failed to get uptime".to_string(),
		};

		let latest_logs_of_the_day = match get_logs_as_string(log_filepath) {
			Ok(logs) => logs,
			Err(error_str) => error_str,
		};

		AdminInfos {
			connected_native_gui_clients,
			connected_web_clients,
			cpu_usage: cpu_usage_avg.load(),
			cpu_temp,
			ram_info,
			uptime,
			latest_logs_of_the_day,
		}
	}
}
