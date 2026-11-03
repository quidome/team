//! Attendance at games (tables `game_attendance` and `game_answers`, story A of attendance at
//! games). An answer needs no lock (decision 0020): each tap is one write, the last tap wins.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use sqlx::{PgConnection, PgExecutor, PgPool};
use team_core::{Answer, ClubNumber, GameAttendance, PlayerId, ReasonId, Season, StoredAnswer};

use crate::db::StoreError;
use crate::duties::GameKey;

fn answer_text(answer: Answer) -> &'static str {
    match answer {
        Answer::Coming => "coming",
        Answer::NotComing => "not_coming",
    }
}

fn parse_answer(text: &str) -> Result<Answer, StoreError> {
    match text {
        "coming" => Ok(Answer::Coming),
        "not_coming" => Ok(Answer::NotComing),
        other => Err(StoreError::Invalid(format!("game answer {other}"))),
    }
}

fn stored_answer(
    player_id: i64,
    answer: &str,
    reason_id: Option<i64>,
) -> Result<StoredAnswer, StoreError> {
    Ok(StoredAnswer {
        player: PlayerId(player_id),
        answer: parse_answer(answer)?,
        reason: reason_id.map(ReasonId),
    })
}

fn attendance_of(
    extra_players: i16,
    answers_for: Option<NaiveDate>,
) -> Result<GameAttendance, StoreError> {
    Ok(GameAttendance {
        extra_players: u8::try_from(extra_players)
            .map_err(|_| StoreError::Invalid(format!("extra players {extra_players}")))?,
        answers_for,
    })
}

/// Saves the answer of `player`. The first answer of the game sets `answers_for` to `date`, the
/// date of the game now; a later answer leaves it, so it never confirms answers from before a move.
pub async fn answer(
    db: &PgPool,
    key: GameKey,
    date: NaiveDate,
    player: PlayerId,
    answer: Answer,
    reason: Option<ReasonId>,
) -> Result<(), StoreError> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "INSERT INTO game_attendance (club_number, season_year, source_id)
         VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE game_attendance SET answers_for = $4
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3
           AND NOT EXISTS (SELECT 1 FROM game_answers a
                           WHERE a.club_number = $1 AND a.season_year = $2 AND a.source_id = $3)",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        date,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO game_answers
             (club_number, season_year, source_id, player_id, answer, reason_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (club_number, season_year, source_id, player_id)
         DO UPDATE SET answer = excluded.answer, reason_id = excluded.reason_id",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        player.0,
        answer_text(answer),
        reason.map(|reason| reason.0),
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Puts `player` back to "no answer".
pub async fn no_answer(
    db: impl PgExecutor<'_>,
    key: GameKey,
    player: PlayerId,
) -> Result<(), StoreError> {
    sqlx::query!(
        "DELETE FROM game_answers
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3 AND player_id = $4",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        player.0,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Sets the number of extra players; the last save wins.
pub async fn set_extra(
    db: impl PgExecutor<'_>,
    key: GameKey,
    extra_players: u8,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO game_attendance (club_number, season_year, source_id, extra_players)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (club_number, season_year, source_id)
         DO UPDATE SET extra_players = excluded.extra_players",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        i16::from(extra_players),
    )
    .execute(db)
    .await?;
    Ok(())
}

/// What is stored for the game of `key`.
pub async fn of_game(
    db: &PgPool,
    key: GameKey,
) -> Result<(GameAttendance, Vec<StoredAnswer>), StoreError> {
    let attendance = sqlx::query!(
        "SELECT extra_players, answers_for FROM game_attendance
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_optional(db)
    .await?
    .map(|row| attendance_of(row.extra_players, row.answers_for))
    .transpose()?
    .unwrap_or_default();
    let answers = sqlx::query!(
        "SELECT player_id, answer, reason_id FROM game_answers
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| stored_answer(row.player_id, &row.answer, row.reason_id))
    .collect::<Result<_, _>>()?;
    Ok((attendance, answers))
}

/// What is stored for the games of a season, by source id.
#[derive(Debug, Clone, Default)]
pub struct SeasonAttendance {
    pub games: BTreeMap<i64, GameAttendance>,
    pub answers: BTreeMap<i64, Vec<StoredAnswer>>,
}

impl SeasonAttendance {
    /// The games with an answer: they stay reachable when they leave the game list.
    pub fn answered(&self) -> BTreeSet<i64> {
        self.answers.keys().copied().collect()
    }
}

/// What is stored for the games of `club` in `season`.
pub async fn of_season(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<SeasonAttendance, StoreError> {
    let mut stored = SeasonAttendance::default();
    for row in sqlx::query!(
        "SELECT source_id, extra_players, answers_for FROM game_attendance
         WHERE club_number = $1 AND season_year = $2",
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?
    {
        stored.games.insert(
            row.source_id,
            attendance_of(row.extra_players, row.answers_for)?,
        );
    }
    for row in sqlx::query!(
        "SELECT source_id, player_id, answer, reason_id FROM game_answers
         WHERE club_number = $1 AND season_year = $2",
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?
    {
        stored
            .answers
            .entry(row.source_id)
            .or_default()
            .push(stored_answer(row.player_id, &row.answer, row.reason_id)?);
    }
    Ok(stored)
}

/// Whether `player` has an answer at any game ("Delete player" is then refused).
pub async fn player_has_answers(
    db: impl PgExecutor<'_>,
    player: PlayerId,
) -> Result<bool, StoreError> {
    let found = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM game_answers WHERE player_id = $1) AS "found!""#,
        player.0,
    )
    .fetch_one(db)
    .await?;
    Ok(found)
}
