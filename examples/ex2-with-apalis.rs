use rust_cronjob_ollama::consts::CRON_EXPRESSION_5_MIN;
use rust_cronjob_ollama::cron_util::create_cron_from_apalis;
use rust_cronjob_ollama::helper::{Category, CronExecutionService};

#[tokio::main]
async fn main() {
    let cron_execution_service = CronExecutionService { category: Category::Joke };
    create_cron_from_apalis(CRON_EXPRESSION_5_MIN, cron_execution_service).await;
}