pub mod consts {
    pub const CRON_EXPRESSION_5_SEC: &str = "0/5 * * * * *";
    pub const CRON_EXPRESSION_2_MIN: &str = "0 */2 * * * *";
    pub const CRON_EXPRESSION_5_MIN: &str = "0 */5 * * * *";
}

pub mod helper {
    use std::future::Future;
    use chrono::{DateTime, Utc};
    use chrono_tz::Tz;

    use crate::ollama_helper::get_joke;

    #[derive(Debug, Clone)]
    pub struct CronArgument {
        pub date_time: DateTime<Tz>,
    }

    impl From<DateTime<Utc>> for CronArgument {
        fn from(t: DateTime<Utc>) -> Self {
            CronArgument {
                date_time: t.with_timezone(&Tz::Japan)
            }
        }
    }

    pub trait TaskExecutor {
        fn execute(&self, argument: CronArgument) -> impl Future<Output=()> + Send;
    }

    #[derive(Clone)]
    pub enum Category {
        Joke,
        Love,
    }

    #[derive(Clone)]
    pub struct CronExecutionService {
        pub category: Category,
    }

    impl TaskExecutor for CronExecutionService {
        async fn execute(&self, argument: CronArgument) {
            match self.category {
                Category::Joke => {
                    println!("Executing joke task at {}", argument.date_time.to_string());
                    let result = get_joke().await;
                    match result {
                        Ok(joke) => println!("{:?}", joke),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Category::Love => {
                    println!("Executing love task at {}", argument.date_time.to_string());
                }
            }
        }
    }
}

pub mod cron_util {
    use std::str::FromStr;
    use std::thread;

    use apalis::layers::retry::{RetryLayer, RetryPolicy};
    use apalis::prelude::{Data, Monitor, TokioExecutor, WorkerBuilder, WorkerFactoryFn};
    use apalis_cron::CronStream;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Asia::Tokyo;
    use cron::Schedule;

    use crate::helper::{CronArgument, CronExecutionService, TaskExecutor};

    pub fn create_cronjob_with_schedule(cron_expression: &str, task: fn()) {
        let schedule = Schedule::from_str(cron_expression)
            .expect("Couldn't start the scheduler. Check the cron expression.");
        loop {
            let utc_now = Utc::now().naive_utc();
            let jst_now = Tokyo.from_utc_datetime(&utc_now);
            if let Some(next) = schedule.upcoming(jst_now.timezone()).take(1).next() {
                let until_next = next - jst_now;
                thread::sleep(until_next.to_std().unwrap());
                println!("Running task at {}", jst_now.to_string());
                task();
            }
        }
    }

    pub async fn create_cron_from_apalis(cron_expression: &str, cron_execution_service: CronExecutionService) {
        let schedule = Schedule::from_str(cron_expression)
            .expect("Couldn't start the scheduler. Check the cron expression.");
        let worker = WorkerBuilder::new("worker")
            .layer(RetryLayer::new(RetryPolicy::retries(3)))
            .stream(CronStream::new(schedule).into_stream())
            .data(cron_execution_service)
            .build_fn(perform_task);
        Monitor::<TokioExecutor>::new()
            .register(worker)
            .run()
            .await
            .unwrap();
    }

    async fn perform_task(job: CronArgument, svc: Data<CronExecutionService>) {
        svc.execute(job).await;
    }
}

pub mod ollama_helper {
    use ollama_rs::generation::completion::request::GenerationRequest;
    use ollama_rs::Ollama;

    pub async fn get_joke() -> Result<String, String> {
        // By default it will connect to localhost:11434
        let ollama = Ollama::default();
        let model = "mistral:latest".to_string();
        let prompt = "I need the most unique, outlandish dad joke you've got. The weirder, the better! Give me 1".to_string();
        let ollama_result = ollama.generate(GenerationRequest::new(model, prompt)).await;
        match ollama_result {
            Ok(generation_response) => Ok(generation_response.response),
            Err(e) => Err(e),
        }
    }
}