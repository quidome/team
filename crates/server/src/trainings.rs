//! Trainings and absences (tables `trainings`, `absences`; decision 0019).

use chrono::NaiveDate;
use sqlx::PgExecutor;
use team_core::{Absence, PlayerId, ReasonId, Season, Training, TrainingId};

use crate::db::StoreError;

struct TrainingRow {
    id: i64,
    date: NaiveDate,
    start_time: chrono::NaiveTime,
    end_time: chrono::NaiveTime,
    cancelled: Option<bool>,
}

impl From<TrainingRow> for Training {
    fn from(row: TrainingRow) -> Self {
        Training {
            id: TrainingId(row.id),
            date: row.date,
            start: row.start_time,
            end: row.end_time,
            cancelled: row.cancelled.unwrap_or(false),
        }
    }
}

/// The trainings of `season`, in order of date.
pub async fn for_season(
    db: impl PgExecutor<'_>,
    season: Season,
) -> Result<Vec<Training>, StoreError> {
    let rows = sqlx::query_as!(
        TrainingRow,
        r#"SELECT id, date, start_time, end_time, cancelled_at IS NOT NULL AS cancelled
           FROM trainings WHERE date BETWEEN $1 AND $2 ORDER BY date"#,
        season.first_day(),
        season.last_day(),
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Training::from).collect())
}

/// The training on `date`, if any.
pub async fn on(db: impl PgExecutor<'_>, date: NaiveDate) -> Result<Option<Training>, StoreError> {
    let row = sqlx::query_as!(
        TrainingRow,
        r#"SELECT id, date, start_time, end_time, cancelled_at IS NOT NULL AS cancelled
           FROM trainings WHERE date = $1"#,
        date,
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(Training::from))
}

/// The absences at the given trainings.
pub async fn absences_at(
    db: impl PgExecutor<'_>,
    trainings: &[TrainingId],
) -> Result<Vec<Absence>, StoreError> {
    let ids: Vec<i64> = trainings.iter().map(|id| id.0).collect();
    let rows = sqlx::query!(
        "SELECT training_id, player_id, reason_id FROM absences WHERE training_id = ANY($1)",
        &ids,
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| Absence {
            training: TrainingId(row.training_id),
            player: PlayerId(row.player_id),
            reason: ReasonId(row.reason_id),
        })
        .collect())
}
