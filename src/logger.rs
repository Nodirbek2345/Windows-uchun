pub fn init_logger() {
    // Initializing simple tracing subscriber
    // In production, log level can be read from config
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();
    
    tracing::info!("Logger initialized.");
}
