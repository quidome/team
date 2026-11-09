//! The report (decision 0033): for each player of a season, the attendance at trainings and games
//! and the duties, from the same counts as the pages "Attendance" (`season_overview`) and "Duties"
//! (`duty_totals`), and the Dutch texts to copy for each column. Names come from the caller.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::contribution::attendance::{PlayerTotals, SeasonOverview};
use crate::contribution::duties::{DutyKind, KindTotals, has_duty_row};
use crate::output::messages::day_and_short_month;
use crate::shared::season::Season;
use crate::team::players::{Group, Player, PlayerId, PlayerRole, place};

/// The attendance of a player at the trainings or the games that count for the player.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Share {
    pub counted: usize,
    pub there: usize,
    pub absent_valid: usize,
    pub absent_not_valid: usize,
}

impl Share {
    /// "67%" of "there".
    pub fn percent_there(&self) -> Option<u8> {
        percent(self.there, self.counted)
    }

    pub fn percent_valid(&self) -> Option<u8> {
        percent(self.absent_valid, self.counted)
    }

    pub fn percent_not_valid(&self) -> Option<u8> {
        percent(self.absent_not_valid, self.counted)
    }
}

/// One row of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow<'a> {
    pub player: &'a Player,
    /// Under "Left the team": no role today or later in the season (also no role at all).
    pub left: bool,
    /// `None`: no training counts for the player ("–").
    pub trainings: Option<Share>,
    /// `None`: no game counts for the player ("–").
    pub games: Option<Share>,
    /// `None` ("–"): no team chosen, or no role "plays games" in the season and no duty.
    pub duties: Option<KindTotals>,
}

/// A column of the report, for its text to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportColumn {
    Trainings,
    Games,
    Drive,
    Table,
    Referee,
}

impl ReportColumn {
    fn heading(self) -> &'static str {
        match self {
            Self::Trainings => "Trainingen",
            Self::Games => "Wedstrijden",
            Self::Drive => "Rijden",
            Self::Table => "Tafel",
            Self::Referee => "Scheidsrechter",
        }
    }

    fn duty(self) -> Option<DutyKind> {
        match self {
            Self::Trainings | Self::Games => None,
            Self::Drive => Some(DutyKind::Drive),
            Self::Table => Some(DutyKind::Table),
            Self::Referee => Some(DutyKind::Referee),
        }
    }
}

/// `part` of `whole` as a whole percentage, half up (2 of 3: 67; 1 of 8: 13); `None` when
/// `whole` is 0.
pub fn percent(part: usize, whole: usize) -> Option<u8> {
    if whole == 0 {
        return None;
    }
    let value = (200 * part + whole) / (2 * whole);
    u8::try_from(value).ok()
}

fn trainings_of(totals: &PlayerTotals<'_>) -> Option<Share> {
    (totals.trainings > 0).then_some(Share {
        counted: totals.trainings,
        there: totals.there,
        absent_valid: totals.absent_valid,
        absent_not_valid: totals.absent_not_valid,
    })
}

fn games_of(totals: &PlayerTotals<'_>) -> Option<Share> {
    let games = totals.games;
    (games.games > 0).then_some(Share {
        counted: games.games,
        there: games.there,
        absent_valid: games.absent_valid,
        absent_not_valid: games.absent_not_valid,
    })
}

/// The rows of the report of `season` on `today` (decision 0033): each player with a training or
/// game that counts (`overview`, from `season_overview`), and each player with a row on "Duties"
/// (`has_duty_row` over `duties`, from `duty_totals`; `None`: no team chosen), sorted by first
/// name.
/// A player is in the team when the page "Team" puts the player there (`place`); every other
/// player is under "Left the team". `roles` holds at least the roles of `season`.
pub fn report_rows<'a>(
    season: Season,
    today: NaiveDate,
    players: &'a [Player],
    roles: &[PlayerRole],
    overview: &SeasonOverview<'a>,
    duties: Option<&BTreeMap<PlayerId, KindTotals>>,
) -> Vec<ReportRow<'a>> {
    let day = today.min(season.last_day());
    let mut sorted: Vec<&Player> = players.iter().collect();
    sorted.sort_by_cached_key(|player| player.sort_key());
    sorted
        .into_iter()
        .filter_map(|player| {
            let attendance = overview
                .team
                .iter()
                .chain(&overview.left)
                .find(|totals| totals.player.id == player.id);
            // The rule of "Duties" (`has_duty_row`); without a team, the role alone.
            let duty_row = match duties {
                Some(totals) => has_duty_row(player.id, roles, season, totals),
                None => has_duty_row(player.id, roles, season, &BTreeMap::new()),
            };
            if attendance.is_none() && !duty_row {
                return None;
            }
            let duties = duties.and_then(|totals| {
                duty_row.then(|| totals.get(&player.id).copied().unwrap_or_default())
            });
            Some(ReportRow {
                player,
                left: place(roles, player.id, season, day).group != Group::Team,
                trainings: attendance.and_then(trainings_of),
                games: attendance.and_then(games_of),
                duties,
            })
        })
        .collect()
}

/// The Dutch text of `column` to copy: a heading with the kind, the club and team (`None`: no team
/// chosen), the season and `today`, then a line for each row in the team (not "Left the team")
/// with a value in the column, by `names`. `None` when no line is left.
pub fn report_copy(
    column: ReportColumn,
    rows: &[ReportRow<'_>],
    names: &BTreeMap<PlayerId, String>,
    club_and_team: Option<(&str, &str)>,
    season: Season,
    today: NaiveDate,
) -> Option<String> {
    let lines: Vec<String> = rows
        .iter()
        .filter(|row| !row.left)
        .filter_map(|row| {
            let value = match column.duty() {
                None => {
                    let share = match column {
                        ReportColumn::Trainings => row.trainings,
                        _ => row.games,
                    };
                    format!("{}%", share?.percent_there()?)
                }
                Some(kind) => {
                    let total = row.duties?.of(kind);
                    if total.planned > 0 {
                        format!("{} (+{} gepland)", total.done, total.planned)
                    } else {
                        total.done.to_string()
                    }
                }
            };
            let name = names
                .get(&row.player.id)
                .cloned()
                .unwrap_or_else(|| row.player.first_name.clone());
            Some(format!("{name} {value}"))
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    let who = match club_and_team {
        Some((club, team)) => format!(" {club} {team},"),
        None => String::new(),
    };
    let heading = format!(
        "{}{who} seizoen {season} (tot en met {})",
        column.heading(),
        day_and_short_month(today)
    );
    Some(
        std::iter::once(heading)
            .chain(lines)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contribution::attendance::GameTotals;
    use crate::team::players::{Role, RoleId};

    fn date(month: u32, day: u32) -> NaiveDate {
        let year = if month >= 7 { 2026 } else { 2027 };
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn season() -> Season {
        Season::from_start_year(2026)
    }

    fn player(id: i64, first_name: &str) -> Player {
        Player {
            id: PlayerId(id),
            first_name: first_name.to_owned(),
            last_name: None,
            hidden: false,
        }
    }

    fn role(player: i64, role: Role, from: NaiveDate, until: Option<NaiveDate>) -> PlayerRole {
        PlayerRole {
            id: RoleId(player),
            player: PlayerId(player),
            season: season(),
            role,
            from,
            until,
        }
    }

    fn totals(player: &Player, trainings: (usize, usize, usize, usize)) -> PlayerTotals<'_> {
        let (counted, there, valid, not_valid) = trainings;
        PlayerTotals {
            player,
            there,
            absent_valid: valid,
            absent_not_valid: not_valid,
            trainings: counted,
            games: GameTotals::default(),
            missed: Vec::new(),
        }
    }

    fn overview<'a>(
        team: Vec<PlayerTotals<'a>>,
        left: Vec<PlayerTotals<'a>>,
    ) -> SeasonOverview<'a> {
        SeasonOverview {
            trainings: 0,
            games: 0,
            team,
            left,
            not_opened: 0,
            first_not_opened: None,
        }
    }

    fn duty(done: u32, planned: u32) -> KindTotals {
        let mut totals = KindTotals::default();
        totals.drive.done = done;
        totals.drive.planned = planned;
        totals
    }

    #[test]
    fn percent_is_rounded_half_up() {
        assert_eq!(percent(2, 3), Some(67));
        assert_eq!(percent(1, 8), Some(13));
        assert_eq!(percent(1, 2), Some(50));
        assert_eq!(percent(9, 9), Some(100));
        assert_eq!(percent(0, 4), Some(0));
        assert_eq!(percent(0, 0), None);
        // 2 + 2 + 1 of 9: 22 + 22 + 11, the parts on their own.
        let share = Share {
            counted: 9,
            there: 6,
            absent_valid: 2,
            absent_not_valid: 1,
        };
        assert_eq!(
            (
                share.percent_there(),
                share.percent_valid(),
                share.percent_not_valid()
            ),
            (Some(67), Some(22), Some(11))
        );
    }

    #[test]
    fn rows_join_attendance_roles_and_duties() {
        // Fien is hidden: hidden players have rows too.
        let players = [
            player(1, "Anna"),
            player(2, "bram"),
            player(3, "Cas"),
            player(4, "Dirk"),
            player(5, "Eva"),
            Player {
                hidden: true,
                ..player(6, "Fien")
            },
        ];
        let roles = [
            // Anna plays games, nothing counted yet.
            role(1, Role::PlaysGames, date(9, 1), None),
            // Bram only trains, with trainings.
            role(2, Role::OnlyTrains, date(9, 1), None),
            // Cas left in October.
            role(3, Role::PlaysGames, date(9, 1), Some(date(10, 15))),
            // Dirk only trains, nothing counted: no row.
            role(4, Role::OnlyTrains, date(9, 1), None),
            // Fien starts in December: in the team.
            role(6, Role::PlaysGames, date(12, 1), None),
        ];
        let attendance = overview(
            vec![totals(&players[1], (3, 2, 1, 0))],
            vec![totals(&players[2], (4, 2, 0, 2))],
        );
        // Eva has a duty and no role in the season.
        let duties = BTreeMap::from([(PlayerId(3), duty(1, 0)), (PlayerId(5), duty(1, 0))]);
        let rows = report_rows(
            season(),
            date(11, 9),
            &players,
            &roles,
            &attendance,
            Some(&duties),
        );
        let ids: Vec<(i64, bool)> = rows.iter().map(|row| (row.player.id.0, row.left)).collect();
        assert_eq!(
            ids,
            [(1, false), (2, false), (3, true), (5, true), (6, false)]
        );
        // Anna: no attendance yet, duties 0 + 0.
        assert_eq!(rows[0].trainings, None);
        assert_eq!(rows[0].duties, Some(KindTotals::default()));
        // Bram only trains: trainings, no games, no duties.
        assert_eq!(rows[1].trainings.map(|share| share.counted), Some(3));
        assert_eq!(rows[1].games, None);
        assert_eq!(rows[1].duties, None);
        // Cas left, with numbers.
        assert_eq!(rows[2].trainings.and_then(|s| s.percent_there()), Some(50));
        assert_eq!(rows[2].duties.map(|d| d.drive.done), Some(1));
        // Eva: a duty without a role.
        assert_eq!(rows[3].duties.map(|d| d.drive.done), Some(1));
    }

    #[test]
    fn a_player_with_only_a_correction_has_no_row_as_on_duties() {
        // Planned, then replaced; no role "plays games" and no attendance (decision 0033).
        let players = [player(1, "Anna")];
        let mut replaced = KindTotals::default();
        replaced.table.done_by_other = 1;
        let duties = BTreeMap::from([(PlayerId(1), replaced)]);
        let attendance = overview(Vec::new(), Vec::new());
        let rows = report_rows(
            season(),
            date(11, 9),
            &players,
            &[],
            &attendance,
            Some(&duties),
        );
        assert_eq!(rows, []);
        // With a training that counts: a row, and no duty numbers, as "Duties" has no row.
        let attendance = overview(Vec::new(), vec![totals(&players[0], (1, 1, 0, 0))]);
        let rows = report_rows(
            season(),
            date(11, 9),
            &players,
            &[],
            &attendance,
            Some(&duties),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].duties, None);
    }

    #[test]
    fn without_a_team_all_duties_are_unknown() {
        let players = [player(1, "Anna")];
        let roles = [role(1, Role::PlaysGames, date(9, 1), None)];
        let attendance = overview(vec![totals(&players[0], (2, 2, 0, 0))], Vec::new());
        let rows = report_rows(season(), date(11, 9), &players, &roles, &attendance, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].duties, None);
        assert_eq!(rows[0].trainings.map(|share| share.there), Some(2));
    }

    #[test]
    fn games_count_from_the_game_totals() {
        let players = [player(1, "Anna")];
        let roles = [role(1, Role::PlaysGames, date(9, 1), None)];
        let mut anna = totals(&players[0], (0, 0, 0, 0));
        anna.games = GameTotals {
            games: 4,
            there: 2,
            absent_valid: 1,
            absent_not_valid: 1,
        };
        let attendance = overview(vec![anna], Vec::new());
        let rows = report_rows(season(), date(11, 9), &players, &roles, &attendance, None);
        assert_eq!(rows[0].trainings, None);
        assert_eq!(
            rows[0].games.and_then(|share| share.percent_there()),
            Some(50)
        );
    }

    #[test]
    fn copy_has_the_team_only_and_no_empty_cells() {
        let players = [player(1, "Anna"), player(2, "Bram"), player(3, "Cas")];
        let roles = [
            role(1, Role::PlaysGames, date(9, 1), None),
            role(2, Role::OnlyTrains, date(9, 1), None),
            role(3, Role::PlaysGames, date(9, 1), Some(date(10, 15))),
        ];
        let attendance = overview(
            vec![
                totals(&players[0], (10, 9, 1, 0)),
                totals(&players[1], (3, 2, 0, 1)),
            ],
            vec![totals(&players[2], (4, 2, 0, 2))],
        );
        let duties = BTreeMap::from([(PlayerId(1), duty(3, 1)), (PlayerId(3), duty(1, 0))]);
        let rows = report_rows(
            season(),
            date(11, 9),
            &players,
            &roles,
            &attendance,
            Some(&duties),
        );
        let names = BTreeMap::from([
            (PlayerId(1), "Anna".to_owned()),
            (PlayerId(2), "Bram".to_owned()),
            (PlayerId(3), "Cas".to_owned()),
        ]);
        let copy = |column, who| report_copy(column, &rows, &names, who, season(), date(11, 9));
        let team = Some(("Blue Drakes", "M16 1"));
        assert_eq!(
            copy(ReportColumn::Trainings, team).as_deref(),
            Some(
                "Trainingen Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)\nAnna 90%\nBram 67%"
            )
        );
        // Bram has no duties ("–"), Cas left.
        assert_eq!(
            copy(ReportColumn::Drive, team).as_deref(),
            Some(
                "Rijden Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)\nAnna 3 (+1 gepland)"
            )
        );
        assert_eq!(
            copy(ReportColumn::Table, None).as_deref(),
            Some("Tafel seizoen 2026-2027 (tot en met 9 nov)\nAnna 0")
        );
        // No game counts for anyone: no text.
        assert_eq!(copy(ReportColumn::Games, team), None);
    }
}
