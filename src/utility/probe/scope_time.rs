use std::time::{Duration, Instant};

pub struct ScopeTime<'a> {
	start: Instant,
	on_drop: Box<dyn FnMut(Instant, Instant) + 'a>,
}

impl<'a> ScopeTime<'a> {
	pub fn new(on_destroy: impl FnMut(Instant, Instant) + 'a) -> Self {
		Self {
			start: Instant::now(),
			on_drop: Box::new(on_destroy),
		}
	}
}

impl<'a> Drop for ScopeTime<'a> {
	fn drop(&mut self) {
		let end = Instant::now();
		(self.on_drop)(self.start, end);
	}
}

#[cfg(test)]
mod tests {
	use tokio::runtime::Builder;

	use super::*;

	#[test]
	fn test_probe_scope() {
		let mut scope_start = Instant::now();
		let mut scope_end = Instant::now();
		let before_scope = Instant::now();
		{
			ScopeTime::new(|start, end| {
				scope_start = start;
				scope_end = end;
			});
			std::thread::sleep(Duration::new(0, 1));  // Sleep for 1 nanosecond
		}
		assert!(scope_start >= before_scope);
		assert!(scope_end > scope_start);

		let delta = scope_end - scope_start;

		assert!(delta >= Duration::new(0, 1));
	}

	#[test]
	fn test_probe_async_scope() {
		let mut duration = Duration::new(0, 0);

		let runtime = Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap();

		runtime.block_on(async {
			ScopeTime::new(|start, end| {
				duration = end - start;
			});
			tokio::time::sleep(Duration::new(0, 1)).await;
		});
		assert!(duration >= Duration::new(0, 1));
	}
}
