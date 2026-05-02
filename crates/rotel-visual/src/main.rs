#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(err) = rotel_visual::run_server().await {
        eprintln!("Fatal: {err}");
        std::process::exit(1);
    }
}
