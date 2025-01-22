use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use systemstat::{Platform, System};

/// number is directly the usage percent number
///
/// `AtomicU32` is used to atomically store a f32 (`f32::as_bits`, `f32::to_bits`)
#[derive(Debug, Default)]
pub struct CpuUsageAverage(pub AtomicU32);

impl CpuUsageAverage {
	pub fn new() -> CpuUsageAverage {
		Self(AtomicU32::new(0))
	}

	pub fn load(&self) -> f32 {
		f32::from_bits(self.0.load(Ordering::Relaxed))
	}

	fn store(&self, percentage: f32) {
		self.0.store(percentage.to_bits(), Ordering::Relaxed);
	}
}

pub async fn messure_cpu_usage_avg_thread(cpu_usage_avg: Arc<CpuUsageAverage>) {
	let sys = System::new();
	loop {
		if let Ok(cpu_load) = sys.cpu_load_aggregate() {
			tokio::time::sleep(std::time::Duration::from_secs(2)).await;
			if let Ok(cpu_load) = cpu_load.done() {
				cpu_usage_avg.store(1.0 - cpu_load.idle);
			}
		}
	}
}
