//! Attendance at games (tables `game_attendance` and `game_answers`; stories A and C of
//! attendance at games). An answer needs no lock (decision 0020): each tap is one write,
//! the last tap wins. "Mark as checked" and its "Undo" lock the row of the game in `game_attendance`.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use sqlx::{PgConnection, PgExecutor, PgPool};
use team_core::{Answer, ClubNumber, GameAttendance, PlayerId, ReasonId, Season, StoredAnswer};

use crate::db::StoreError;
use crate::games::GameKey;

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
    settled: bool,
) -> Result<StoredAnswer, StoreError> {
    Ok(StoredAnswer {
        player: PlayerId(player_id),
        answer: parse_answer(answer)?,
        reason: reason_id.map(ReasonId),
        settled,
    })
}

fn attendance_of(
    extra_players: i16,
    answers_for: Option<NaiveDate>,
    checked_for: Option<NaiveDate>,
) -> Result<GameAttendance, StoreError> {
    Ok(GameAttendance {
        extra_players: u8::try_from(extra_players)
            .map_err(|_| StoreError::Invalid(format!("extra players {extra_players}")))?,
        answers_for,
        checked_for,
    })
}

/// Saves the answer of `player`. The first answer of the game sets `answers_for` to `date`, the
/// date of the game now; a later answer leaves it, so it never confirms answers from before a move.
/// An answer that "Checked" made is not a first answer: it holds only while the check holds.
/// A tap is never settled: it clears `settled`, so the "Undo" of a check keeps it.
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
                           WHERE a.club_number = $1 AND a.season_year = $2 AND a.source_id = $3
                             AND NOT a.settled)",
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
         DO UPDATE SET answer = excluded.answer, reason_id = excluded.reason_id, settled = false",
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

/// "Mark answers as checked": the answers hold for `date`, the date of the game now. One write; a second
/// tap writes the same date.
pub async fn answers_checked(
    db: impl PgExecutor<'_>,
    key: GameKey,
    date: NaiveDate,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO game_attendance (club_number, season_year, source_id, answers_for)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (club_number, season_year, source_id)
         DO UPDATE SET answers_for = excluded.answers_for",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        date,
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
    let mut conn = db.acquire().await?;
    read(&mut conn, key).await
}

/// What is stored for the game of `key`, read on `db` (also inside a transaction).
async fn read(
    db: &mut PgConnection,
    key: GameKey,
) -> Result<(GameAttendance, Vec<StoredAnswer>), StoreError> {
    let attendance = sqlx::query!(
        "SELECT extra_players, answers_for, checked_for FROM game_attendance
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_optional(&mut *db)
    .await?
    .map(|row| attendance_of(row.extra_players, row.answers_for, row.checked_for))
    .transpose()?
    .unwrap_or_default();
    let answers = sqlx::query!(
        "SELECT player_id, answer, reason_id, settled FROM game_answers
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_all(&mut *db)
    .await?
    .into_iter()
    .map(|row| stored_answer(row.player_id, &row.answer, row.reason_id, row.settled))
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
        "SELECT source_id, extra_players, answers_for, checked_for FROM game_attendance
         WHERE club_number = $1 AND season_year = $2",
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?
    {
        stored.games.insert(
            row.source_id,
            attendance_of(row.extra_players, row.answers_for, row.checked_for)?,
        );
    }
    for row in sqlx::query!(
        "SELECT source_id, player_id, answer, reason_id, settled FROM game_answers
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
            .push(stored_answer(
                row.player_id,
                &row.answer,
                row.reason_id,
                row.settled,
            )?);
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

/// Locks the row of the game of `key` for a check or its "Undo", inserting it first when it
/// is missing; gives its `checked_for`. Only inside the transaction of the write.
async fn lock(tx: &mut PgConnection, key: GameKey) -> Result<Option<NaiveDate>, StoreError> {
    sqlx::query!(
        "INSERT INTO game_attendance (club_number, season_year, source_id)
         VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .execute(&mut *tx)
    .await?;
    let checked_for = sqlx::query_scalar!(
        "SELECT checked_for FROM game_attendance
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3
         FOR UPDATE",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    Ok(checked_for)
}

/// "Checked" on `date`, the date of the game now, under the lock of the game. `to_settle` gets
/// what is stored, read under the lock, and gives the players with "no answer"; each gets a
/// settled "not coming" without a reason; a tap at the same moment wins (`ON CONFLICT DO
/// NOTHING`). Gives the number of those players, also after a move, when their settled rows
/// of the old date already exist (S41). A game already checked for `date` changes nothing: 0.
pub async fn check(
    tx: &mut PgConnection,
    key: GameKey,
    date: NaiveDate,
    to_settle: impl FnOnce(&GameAttendance, &[StoredAnswer]) -> Vec<PlayerId>,
) -> Result<usize, StoreError> {
    if lock(tx, key).await? == Some(date) {
        return Ok(0);
    }
    let (attendance, answers) = read(tx, key).await?;
    let players = to_settle(&attendance, &answers);
    sqlx::query!(
        "UPDATE game_attendance SET checked_for = $4
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        date,
    )
    .execute(&mut *tx)
    .await?;
    let ids: Vec<i64> = players.iter().map(|player| player.0).collect();
    sqlx::query!(
        "INSERT INTO game_answers
             (club_number, season_year, source_id, player_id, answer, settled)
         SELECT $1, $2, $3, player, 'not_coming', true FROM UNNEST($4::bigint[]) AS player
         ON CONFLICT (club_number, season_year, source_id, player_id) DO NOTHING",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        &ids,
    )
    .execute(&mut *tx)
    .await?;
    Ok(players.len())
}

/// The "Undo" of a check under the lock of the game: deletes the answers that "Checked" made and that were
/// not tapped after that, and the check.
pub async fn uncheck(tx: &mut PgConnection, key: GameKey) -> Result<(), StoreError> {
    lock(tx, key).await?;
    sqlx::query!(
        "DELETE FROM game_answers
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3 AND settled",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE game_attendance SET checked_for = NULL
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}
