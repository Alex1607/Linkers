mod cleaner;
mod error;
mod pro_api;
mod providers;
mod utils_api;

#[tokio::main]
async fn main() {
    let mut interval_timer = tokio::time::interval(
        chrono::Duration::seconds(60)
            .to_std()
            .expect("Unable to build interval timer. Bot won't start."),
    );

    loop {
        interval_timer.tick().await;

        tokio::spawn(async {
            let run_result = cleaner::run_linkers().await;
            if let Err(error) = run_result {
                println!("Linkers wasn't able to run. Error: {}", error)
            };
        });
    }
}
