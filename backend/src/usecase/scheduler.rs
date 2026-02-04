use crate::domain::stack_repository::StackRepository;
use crate::error::Result;
use crate::usecase::stack::StackUsecase;
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

pub struct AutomationScheduler {
    stack_usecase: Arc<StackUsecase>,
    stack_repo: Arc<dyn StackRepository>,
}

impl AutomationScheduler {
    pub fn new(stack_usecase: Arc<StackUsecase>, stack_repo: Arc<dyn StackRepository>) -> Self {
        Self {
            stack_usecase,
            stack_repo,
        }
    }

    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting Automation Scheduler...");
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            if let Err(e) = self.check_and_run_jobs().await {
                tracing::error!("Error in automation scheduler loop: {}", e);
            }
        }
    }

    async fn check_and_run_jobs(&self) -> Result<()> {
        let all_stacks = self.stack_repo.list_all().await?;
        let now = Utc::now();

        for stack in all_stacks {
            let schedule_str = match &stack.cron_schedule {
                Some(s) if !s.is_empty() => s,
                _ => continue,
            };

            let schedule = match Schedule::from_str(schedule_str) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Invalid cron schedule for stack {}: {}", stack.id, e);
                    continue;
                }
            };

            let last_minute = now - Duration::from_secs(61);
            let next_occurrence = schedule.after(&last_minute).next();

            if let Some(occ) = next_occurrence
                && occ <= now
            {
                tracing::info!("Triggering scheduled redeploy for stack {}", stack.id);
                let stack_id = stack.id.clone();
                let usecase = self.stack_usecase.clone();

                tokio::spawn(async move {
                    if let Err(e) = usecase.redeploy_stack(&stack_id).await {
                        tracing::error!("Scheduled redeploy failed for stack {}: {}", stack_id, e);
                    }
                });
            }
        }

        Ok(())
    }
}
