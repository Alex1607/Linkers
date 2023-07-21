mod cleaner;
mod pro_api;
mod providers;

#[tokio::main]
async fn main() {
    let mut interval_timer = tokio::time::interval(chrono::Duration::seconds(10).to_std().unwrap());

    loop {
        interval_timer.tick().await;

        tokio::spawn(async {
            cleaner::run_linkers().await;
        });
    }
}
