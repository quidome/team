//! Trainings and absences (tables `trainings`, `absences`; decision 0020).

use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::{PgExecutor, PgPool};
use team_core::{Absence, PlayerId, ReasonId, Season, Training, TrainingId};

use crate::db::StoreError;

struct TrainingRow {
    id: i64,
    date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
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

/// The seasons with a role, a weekly training or a training, oldest first. The season of a
/// training is `Season::of` its date.
pub async fn seasons(db: &PgPool) -> Result<Vec<Season>, StoreError> {
    let years = sqlx::query_scalar!(
        r#"SELECT season_year AS "year!" FROM player_roles
           UNION SELECT season_year FROM weekly_trainings"#
    )
    .fetch_all(db)
    .await?;
    let dates = sqlx::query_scalar!("SELECT DISTINCT date FROM trainings")
        .fetch_all(db)
        .await?;
    let seasons: BTreeSet<Season> = years
        .into_iter()
        .map(Season::from_start_year)
        .chain(dates.into_iter().map(Season::of))
        .collect();
    Ok(seasons.into_iter().collect())
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

/// Opens the training day `date` with the given times: a new training, unless one exists on the
/// date (then nothing changes). Safe to send twice.
pub async fn open(
    db: impl PgExecutor<'_>,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO trainings (date, start_time, end_time, created_at) VALUES ($1, $2, $3, $4)
         ON CONFLICT (date) DO NOTHING",
        date,
        start,
        end,
        now,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// A new training ("Add training"). A date that has a training fails on the unique index
/// (`StoreError::is_unique_violation`).
pub async fn insert(
    db: impl PgExecutor<'_>,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    now: DateTime<Utc>,
) -> Result<TrainingId, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO trainings (date, start_time, end_time, created_at) VALUES ($1, $2, $3, $4)
         RETURNING id",
        date,
        start,
        end,
        now,
    )
    .fetch_one(db)
    .await?;
    Ok(TrainingId(id))
}

/// Changes the date and times of a training ("Change"); `false` when it does not exist. A date that
/// has a training fails on the unique index.
pub async fn update(
    db: impl PgExecutor<'_>,
    id: TrainingId,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE trainings SET date = $2, start_time = $3, end_time = $4 WHERE id = $1",
        id.0,
        date,
        start,
        end,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Cancels the training on `date`; a training day without a training gets a cancelled training
/// with the given times. A training that is already cancelled keeps its first moment.
pub async fn cancel(
    db: impl PgExecutor<'_>,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO trainings (date, start_time, end_time, cancelled_at, created_at)
         VALUES ($1, $2, $3, $4, $4)
         ON CONFLICT (date) DO UPDATE
         SET cancelled_at = coalesce(trainings.cancelled_at, excluded.cancelled_at)",
        date,
        start,
        end,
        now,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Makes a cancelled training a normal training again ("Undo cancel"); `false` when there is no
/// training on `date`.
pub async fn uncancel(db: impl PgExecutor<'_>, date: NaiveDate) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE trainings SET cancelled_at = NULL WHERE date = $1",
        date
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Deletes the training on `date` with its absences; `false` when there is none.
pub async fn delete(db: impl PgExecutor<'_>, date: NaiveDate) -> Result<bool, StoreError> {
    let result = sqlx::query!("DELETE FROM trainings WHERE date = $1", date)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Marks `player` absent at `training` with `reason`; the last mark wins.
pub async fn mark(
    db: impl PgExecutor<'_>,
    training: TrainingId,
    player: PlayerId,
    reason: ReasonId,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO absences (training_id, player_id, reason_id) VALUES ($1, $2, $3)
         ON CONFLICT (training_id, player_id) DO UPDATE SET reason_id = excluded.reason_id",
        training.0,
        player.0,
        reason.0,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Marks `player` there again at `training`: its absence goes, if any.
pub async fn unmark(
    db: impl PgExecutor<'_>,
    training: TrainingId,
    player: PlayerId,
) -> Result<(), StoreError> {
    sqlx::query!(
        "DELETE FROM absences WHERE training_id = $1 AND player_id = $2",
        training.0,
        player.0,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Whether `player` has an absence at any training ("Delete player" is then refused).
pub async fn player_has_absences(
    db: impl PgExecutor<'_>,
    player: PlayerId,
) -> Result<bool, StoreError> {
    let found = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM absences WHERE player_id = $1) AS "found!""#,
        player.0
    )
    .fetch_one(db)
    .await?;
    Ok(found)
}
