#[macro_export]
macro_rules! log_scope_time {
    () => {
        let _time = crate::utility::ScopeTime::new(|start, end| {
            log::info!("{:#?} -> {:#?}", start, end)
        });
    };
}
