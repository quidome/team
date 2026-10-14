//! The nightly reading (story 5): a timer in the app reads the game list once a night, instead of
//! the Kubernetes CronJob of decision 0005. The rules are in `team_core::nightly`.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use team_core::nightly_due;
use tokio::time::MissedTickBehavior;

use crate::clock::{Clock, in_netherlands};
use crate::reading::{self, Mode, ReadingError};
use crate::readings::{self, Reading, ReadingResult};
use crate::source::Source;

/// One check: reads the game list when a nightly reading is due, with the guard on, as a tap
/// does. `None` when no reading is due: no team is chosen, or a reading started since the last
/// run moment.
///
/// The check is outside the reading lock. Two pods during a rollout can both read; the lock and
/// the rule that a later good reading wins keep the data correct.
pub async fn tick(
    pool: &PgPool,
    source: &Source,
    clock: &dyn Clock,
) -> Result<Option<Reading>, ReadingError> {
    let Some(target) = reading::target(pool, clock).await? else {
        return Ok(None);
    };
    let newest_start = readings::newest_start(pool, target.club, target.season).await?;
    let now = in_netherlands(clock.now());
    if !nightly_due(target.team.is_some(), now, newest_start.map(in_netherlands)) {
        return Ok(None);
    }
    reading::read(pool, source, clock, Mode::Guarded)
        .await
        .map(Some)
}

/// How often the timer checks whether a nightly reading is due.
const CHECK_EVERY: Duration = Duration::from_secs(5 * 60);

/// Calls [`tick`] at once and then every 5 minutes, until the task is stopped. A pod that starts
/// after 03:00 reads at once when the night was missed. Errors are logged and the loop goes on;
/// a failed reading is stored, so it is not tried again the same night.
pub async fn run(pool: PgPool, source: Arc<Source>, clock: Arc<dyn Clock>) {
    let mut interval = tokio::time::interval(CHECK_EVERY);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        interval.tick().await;
        match tick(&pool, &source, clock.as_ref()).await {
            Ok(Some(reading)) => {
                let (result, games_read) = match reading.result {
                    ReadingResult::Good { games_read, .. } => ("good", Some(games_read)),
                    ReadingResult::HeldBack(_) => ("held_back", None),
                    ReadingResult::Failed(_) => ("failed", None),
                };
                tracing::info!(result, games_read, "nightly reading");
            }
            Ok(None) => {}
            Err(error) => tracing::error!(%error, "nightly reading"),
        }
    }
}
