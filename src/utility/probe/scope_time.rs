use std::time::Instant;

pub trait Callback: FnOnce(Instant, Instant) {}

impl<T> Callback for T where T: FnOnce(Instant, Instant) {}

struct ScopeTimeState<F>
where
    F: Callback,
{
    start: Instant,
    on_drop: F,
}

pub struct ScopeTime<F>
where
    F: Callback,
{
    state: Option<ScopeTimeState<F>>,
}

impl<F> ScopeTime<F>
where
    F: Callback,
{
    #[must_use]
    pub fn new(on_drop: F) -> Self {
        Self {
            state: Some(ScopeTimeState {
                start: Instant::now(),
                on_drop,
            }),
        }
    }
}

impl<F> Drop for ScopeTime<F>
where
    F: Callback,
{
    fn drop(&mut self) {
        let end = Instant::now();
        if let Some(state) = self.state.take() {
            (state.on_drop)(state.start, end);
        }
    }
}

// https://doc.rust-lang.org/rust-by-example/macros.html
#[macro_export]
macro_rules! log_scope_time {
    () => {
        log_scope_time!("Probe");
    };
    ($prefix:expr) => {
        let _time = crate::utility::ScopeTime::new(|start, end| {
            log::info!("{} duration: {:?}", $prefix, end - start)
        });
    };
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::Rng;
    use tokio::runtime::Builder;

    use super::*;

    macro_rules! jitter {
        () => {
            // For some number of iterations between 100 and 1000...
            for _ in 0..rand::thread_rng().gen_range(100..1000) {
                // ... loop indefinitely ...
                loop {
                    // ... until randomly choose to break.
                    if rand::random() {
                        break;
                    };
                }
            }
        };
    }

    const ITERATIONS: usize = 15;

    #[test]
    fn test_probe_scope() {
        for _ in 0..ITERATIONS {
            jitter!();

            let mut scope_start = Instant::now();
            let mut scope_end = scope_start.clone();

            std::thread::sleep(Duration::from_nanos(1));

            let before_scope = Instant::now();
            {
                let _probe = ScopeTime::new(|start, end| {
                    scope_start = start;
                    scope_end = end;
                });
                std::thread::sleep(Duration::from_nanos(1));
            }

            assert!(scope_start >= before_scope);
            assert!(scope_end > scope_start);

            let delta = scope_end - scope_start;
            assert!(delta >= Duration::from_nanos(1));
        }
    }

    #[test]
    fn test_probe_async_scope() {
        let mut duration = Duration::new(0, 0);

        let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

        for _ in 0..ITERATIONS {
            jitter!();

            runtime.block_on(async {
                let _probe = ScopeTime::new(|start, end| {
                    duration = end - start;
                });
                tokio::time::sleep(Duration::from_nanos(1)).await;
            });
            assert!(
                duration >= Duration::from_nanos(1),
                "duration was {:#?}",
                duration
            );
        }
    }
}
