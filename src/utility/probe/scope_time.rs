use std::time::{Duration, Instant};

pub struct ScopeTime<'a> {
	origin: Instant,
	on_drop: Box<dyn FnMut(Duration) + 'a>,
}

impl<'a> ScopeTime<'a> {
	pub fn new(on_destroy: impl FnMut(Duration) + 'a) -> Self {
		Self {
			origin: Instant::now(),
			on_drop: Box::new(on_destroy),
		}
	}
}

impl<'a> Drop for ScopeTime<'a> {
	fn drop(&mut self) {
		(self.on_drop)(self.origin.elapsed());
	}
}

#[cfg(test)]
mod tests {
	use tokio::runtime::Builder;

	use super::*;

	#[test]
	fn test_probe_scope() {
		let mut duration = Duration::new(0, 0);
		{
			ScopeTime::new(|d| duration = d);
			std::thread::sleep(Duration::new(0, 1));  // Sleep for 1 nanosecond
		}
		assert!(duration > Duration::new(0, 0));
	}

	#[test]
	fn test_probe_async_scope() {
		let mut duration = Duration::new(0, 0);

		let runtime = Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap();

		runtime.block_on(async {
			ScopeTime::new(|d| duration = d);
			tokio::time::sleep(Duration::new(0, 1)).await;
		});

		assert!(duration > Duration::new(0, 0));
	}
}
