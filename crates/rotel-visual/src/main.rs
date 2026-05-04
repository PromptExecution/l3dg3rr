#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match rotel_visual::run_server().await {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("Fatal: {err}");
            std::process::exit(1);
        }
    }
}
