use crate::utility::probe::ScopeTime;

macro_rules! log_scope_time {
    () => {
        let _time = ScopeTime::new(|duration| log::info!("{:#?}", duration));
    };
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_log_probe_scope() {
		log_scope_time!();
	}

	#[test]
	fn test_log_probe_scope_twice() {
		log_scope_time!();
		log_scope_time!();
	}
}
