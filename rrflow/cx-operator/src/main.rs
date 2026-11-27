#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    if let Err(error) = rrflow_cx_operator::controller::run().await {
        tracing::error!(error = %error, "RRD Kubernetes operator stopped");
        std::process::exit(2);
    }
}
