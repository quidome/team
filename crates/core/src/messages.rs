//! The messages for the coordinator to copy into the chat (stories C and D of duties): Dutch text, made
//! from what is stored at the moment of the request. The tool never sends them.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate, NaiveTime};

use crate::club::ClubNumber;
use crate::duties::{Assignment, Counted, DutyKind, DutyPlace, GamePlaces, SchedulePart, who_did};
use crate::game::{Game, TeamNumber};
use crate::players::{Player, PlayerId};
use crate::schedule::{ScheduleReason, ScheduledGame};

/// A game of the season with its duties, as the messages read it.
#[derive(Debug, Clone, Copy)]
pub struct MessageGame<'a> {
    pub scheduled: &'a ScheduledGame,
    /// `game_places` of the game; `None`: no duties.
    pub places: Option<&'a GamePlaces>,
    pub assignments: &'a [Assignment],
}

const DAYS: [&str; 7] = [
    "maandag",
    "dinsdag",
    "woensdag",
    "donderdag",
    "vrijdag",
    "zaterdag",
    "zondag",
];
const SHORT_DAYS: [&str; 7] = ["ma", "di", "wo", "do", "vr", "za", "zo"];
const MONTHS: [&str; 12] = [
    "januari",
    "februari",
    "maart",
    "april",
    "mei",
    "juni",
    "juli",
    "augustus",
    "september",
    "oktober",
    "november",
    "december",
];
const SHORT_MONTHS: [&str; 12] = [
    "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
];

const TIME_NOT_KNOWN: &str = "tijd nog niet bekend";
const NOBODY: &str = "niemand";

/// Monday and Sunday of the week that holds `date`.
pub fn week_of(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let days_since_monday = u64::from(date.weekday().num_days_from_monday());
    let monday = date
        .checked_sub_days(Days::new(days_since_monday))
        .unwrap_or(date);
    let sunday = monday.checked_add_days(Days::new(6)).unwrap_or(monday);
    (monday, sunday)
}

/// "31 oktober".
fn day_and_month(date: NaiveDate) -> String {
    format!("{} {}", date.day(), MONTHS[date.month0() as usize])
}

/// "26 oktober – 1 november": the week that holds `today`.
fn week_dates(today: NaiveDate) -> String {
    let (monday, sunday) = week_of(today);
    format!("{} – {}", day_and_month(monday), day_and_month(sunday))
}

/// "zaterdag 31 oktober".
fn long_date(date: NaiveDate) -> String {
    let day = DAYS[date.weekday().num_days_from_monday() as usize];
    format!("{day} {}", day_and_month(date))
}

/// "za 31 okt".
fn short_date(date: NaiveDate) -> String {
    let day = SHORT_DAYS[date.weekday().num_days_from_monday() as usize];
    let month = SHORT_MONTHS[date.month0() as usize];
    format!("{day} {} {month}", date.day())
}

fn clock(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

/// The names on `places` (in their order) and the open places: "Anna, Bram", "Eva, nog 1 open",
/// "nog 2 open". `past`: the game is before today, so a correction shows (`who_did`), and a place
/// that stayed open was done by nobody.
fn filled_and_open(
    places: &[&DutyPlace],
    assignments: &[Assignment],
    names: &BTreeMap<PlayerId, String>,
    past: bool,
) -> String {
    let filled: Vec<&str> = places
        .iter()
        .filter(|place| place.filled || past)
        .map(|place| {
            if !place.filled {
                return NOBODY;
            }
            let assignment = assignments
                .iter()
                .find(|a| a.kind == place.kind && a.number == place.number);
            match assignment.map(|a| who_did(a, past)) {
                Some(Counted::Done(player) | Counted::Planned(player)) => {
                    names.get(&player).map_or("?", String::as_str)
                }
                Some(Counted::Nobody) => NOBODY,
                None => "?",
            }
        })
        .collect();
    let open = places.len() - filled.len();
    let names = filled.join(", ");
    match (filled.is_empty(), open) {
        (_, 0) => names,
        (true, open) => format!("nog {open} open"),
        (false, open) => format!("{names}, nog {open} open"),
    }
}

/// The block of one game of the team, with its drivers. "Thuis": a game in the club's own hall
/// (`docs/language.md`), also with the team as the away side against another club team.
fn game_block(
    message_game: &MessageGame<'_>,
    names: &BTreeMap<PlayerId, String>,
    club: ClubNumber,
    team: TeamNumber,
) -> String {
    let game = &message_game.scheduled.game;
    let time = game.start_time.map_or(TIME_NOT_KNOWN.to_owned(), clock);
    let side = if game.home.club == club {
        "Thuis"
    } else {
        "Uit"
    };
    let opponent = if game.home.team == team {
        &game.away.team_name
    } else {
        &game.home.team_name
    };
    let mut lines = vec![
        format!("Wedstrijd: {}, {time}", long_date(game.date)),
        format!("{side} tegen {opponent}"),
        format!("{}, {}", game.place.hall, game.place.town),
    ];
    let drivers = message_game
        .places
        .and_then(|places| places.drivers.as_ref())
        .filter(|part| part.counts && !part.places.is_empty());
    if let Some(part) = drivers {
        let places: Vec<&DutyPlace> = part.places.iter().collect();
        lines.push(format!(
            "Rijden: {}",
            filled_and_open(&places, message_game.assignments, names, false)
        ));
    }
    lines.join("\n")
}

/// The table and referee places of the team at `game`, when they count.
fn places_of_team<'a>(game: &MessageGame<'a>) -> Option<&'a SchedulePart> {
    game.places
        .and_then(|places| places.schedule.as_ref())
        .filter(|part| part.counts && !part.of_team.is_empty())
}

/// The lines of one game with table or referee places of the team, with `names` for the players;
/// `None`: no such places that count. Places of other club teams are never in it.
fn duty_lines(
    game: &MessageGame<'_>,
    names: &BTreeMap<PlayerId, String>,
    today: NaiveDate,
) -> Option<String> {
    let part = places_of_team(game)?;
    let Game {
        date,
        start_time,
        home,
        away,
        ..
    } = &game.scheduled.game;
    let teams = format!("{} – {}", home.team_name, away.team_name);
    let mut lines = vec![match start_time {
        Some(time) => format!("{} {} {teams}", short_date(*date), clock(*time)),
        None => format!("{}, {TIME_NOT_KNOWN}, {teams}", short_date(*date)),
    }];
    for (kind, label) in [
        (DutyKind::Referee, "Scheidsrechter"),
        (DutyKind::Table, "Tafel"),
    ] {
        let places: Vec<&DutyPlace> = part.of_team.iter().filter(|p| p.kind == kind).collect();
        if !places.is_empty() {
            lines.push(format!(
                "  {label}: {}",
                filled_and_open(&places, game.assignments, names, *date < today)
            ));
        }
    }
    Some(lines.join("\n"))
}

/// The games of `games` in the week that holds `today`, in date order.
fn week_games<'g, 'a>(games: &'g [MessageGame<'a>], today: NaiveDate) -> Vec<&'g MessageGame<'a>> {
    let (monday, sunday) = week_of(today);
    let mut week: Vec<&MessageGame<'_>> = games
        .iter()
        .filter(|game| (monday..=sunday).contains(&game.scheduled.game.date))
        .collect();
    week.sort_by_key(|game| game.scheduled.game.date);
    week
}

/// The message of the week that holds `today`. `games`: the games of the season, in time order
/// within a date, the dates in any order; `names`: `display_names` of the season; `club` and
/// `team`: the club and the team chosen for the season. A game of the team has a block from today
/// to Sunday, while it is in the game list; the table and referee places of the team that count
/// are in it for the whole week. `None`: no such game, and no such places of today or later or
/// with a player: a past place that stayed open announces nothing.
pub fn week_message(
    games: &[MessageGame<'_>],
    names: &BTreeMap<PlayerId, String>,
    club: ClubNumber,
    team: TeamNumber,
    today: NaiveDate,
) -> Option<String> {
    let week = week_games(games, today);

    let blocks: Vec<String> = week
        .iter()
        .filter(|game| {
            game.scheduled.reason == ScheduleReason::TeamGame
                && game.scheduled.in_list
                && game.scheduled.game.date >= today
        })
        .map(|game| game_block(game, names, club, team))
        .collect();
    let duties: Vec<String> = week
        .iter()
        .filter_map(|game| duty_lines(game, names, today))
        .collect();
    let places_to_announce = week.iter().any(|game| {
        places_of_team(game).is_some_and(|part| {
            game.scheduled.game.date >= today || part.of_team.iter().any(|place| place.filled)
        })
    });
    if blocks.is_empty() && !places_to_announce {
        return None;
    }

    let mut parts = vec![format!("Week {}", week_dates(today))];
    parts.extend(blocks);
    if !duties.is_empty() {
        parts.push(format!("Tafel en scheidsrechter:\n{}", duties.join("\n\n")));
    }
    Some(parts.join("\n\n"))
}

/// First and last name of `players`; a player without a last name: the first name.
pub fn full_names(players: &[Player]) -> BTreeMap<PlayerId, String> {
    players
        .iter()
        .map(|player| {
            let name = match &player.last_name {
                Some(last) => format!("{} {last}", player.first_name),
                None => player.first_name.clone(),
            };
            (player.id, name)
        })
        .collect()
}

/// The committee message of the week that holds `today`. `games`: as [`week_message`]; `names`:
/// `full_names` of the season; `club_name` and `team_name`: "Blue Drakes" and "M16 1". It always
/// gives a text: the table and referee places of the team that count, or that there are none.
pub fn committee_message(
    games: &[MessageGame<'_>],
    names: &BTreeMap<PlayerId, String>,
    club_name: &str,
    team_name: &str,
    today: NaiveDate,
) -> String {
    let mut parts = vec![format!(
        "Tafel en scheidsrechters {club_name} {team_name}, week {}",
        week_dates(today)
    )];
    parts.extend(
        week_games(games, today)
            .iter()
            .filter_map(|game| duty_lines(game, names, today)),
    );
    if parts.len() == 1 {
        parts.push("Geen tafel- of scheidsrechtersdiensten deze week.".to_owned());
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::duties::{
        Outcome, PlaceKind, StoredDuties, TeamRef, display_names, game_places, required_duties,
    };
    use crate::game::{Place, PlaceNumber, Side};
    use crate::players::Player;
    use crate::schedule::tests::{CLUB, CLUB_TEAM, OPPONENT, OTHER_CLUB, TEAM};
    use crate::season::Season;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn time(hour: u32, minute: u32) -> Option<NaiveTime> {
        NaiveTime::from_hms_opt(hour, minute, 0)
    }

    /// Monday 26 October 2026.
    fn monday() -> NaiveDate {
        date(2026, 10, 26)
    }

    fn side(team: TeamNumber, name: &str, club: ClubNumber) -> Side {
        Side {
            team,
            team_name: name.to_owned(),
            club,
        }
    }

    const ANNA: PlayerId = PlayerId(1);
    const BRAM: PlayerId = PlayerId(2);
    const CAS: PlayerId = PlayerId(3);
    const EVA: PlayerId = PlayerId(5);
    const FLEUR: PlayerId = PlayerId(6);
    const GIJS: PlayerId = PlayerId(7);

    fn names() -> BTreeMap<PlayerId, String> {
        [
            (ANNA, "Anna"),
            (BRAM, "Bram"),
            (CAS, "Cas"),
            (EVA, "Eva"),
            (FLEUR, "Fleur"),
            (GIJS, "Gijs"),
        ]
        .into_iter()
        .map(|(id, name)| (id, name.to_owned()))
        .collect()
    }

    /// A game with its stored duties and players, before `game_places`.
    struct Fixture {
        scheduled: ScheduledGame,
        stored: StoredDuties,
        assignments: Vec<Assignment>,
    }

    impl Fixture {
        fn game(
            id: i64,
            on: NaiveDate,
            start: Option<NaiveTime>,
            home: Side,
            away: Side,
            reason: ScheduleReason,
        ) -> Self {
            Self {
                scheduled: ScheduledGame {
                    game: Game {
                        source_id: id,
                        game_number: format!("N{id}"),
                        season: Season::of(on),
                        date: on,
                        start_time: start,
                        home,
                        away,
                        place: Place {
                            number: PlaceNumber(1),
                            hall: "Sporthal Bulwijk".to_owned(),
                            town: "Woerden".to_owned(),
                            court: Some("A".to_owned()),
                        },
                        competition: "Competition".to_owned(),
                    },
                    reason,
                    in_list: true,
                },
                stored: StoredDuties::default(),
                assignments: Vec::new(),
            }
        }

        /// An away game of the team at another club.
        fn away(id: i64, on: NaiveDate, start: Option<NaiveTime>) -> Self {
            Self::game(
                id,
                on,
                start,
                side(OPPONENT, "B.Z. '72 M16 1", OTHER_CLUB),
                side(TEAM, "Blue Drakes M16 1", CLUB),
                ScheduleReason::TeamGame,
            )
        }

        /// A home game of the team.
        fn home(id: i64, on: NaiveDate, start: Option<NaiveTime>) -> Self {
            Self::game(
                id,
                on,
                start,
                side(TEAM, "Blue Drakes M16 1", CLUB),
                side(OPPONENT, "Lions M16 1", OTHER_CLUB),
                ScheduleReason::TeamGame,
            )
        }

        /// A home game of another club team.
        fn duty(id: i64, on: NaiveDate, start: Option<NaiveTime>) -> Self {
            Self::game(
                id,
                on,
                start,
                side(CLUB_TEAM, "Blue Drakes X12 1", CLUB),
                side(TeamNumber(901), "De Dunckers X12 1", OTHER_CLUB),
                ScheduleReason::DutyGame,
            )
        }

        fn drivers(mut self, number: u8) -> Self {
            self.stored.drivers = Some(number);
            self
        }

        /// The place names the team.
        fn of_team(mut self, kind: PlaceKind, number: u8) -> Self {
            let team = TeamRef {
                number: TEAM,
                name: "M16 1".to_owned(),
            };
            self.stored.teams.insert((kind, number), team);
            self
        }

        fn player(mut self, kind: DutyKind, number: u8, player: PlayerId) -> Self {
            self.assignments.push(Assignment {
                kind,
                number,
                planned: player,
                outcome: Outcome::AsPlanned,
            });
            self
        }

        /// The place names another club team.
        fn of_other_team(mut self, kind: PlaceKind, number: u8) -> Self {
            let team = TeamRef {
                number: CLUB_TEAM,
                name: "X12 1".to_owned(),
            };
            self.stored.teams.insert((kind, number), team);
            self
        }

        /// The last player added got `outcome`.
        fn outcome(mut self, outcome: Outcome) -> Self {
            self.assignments
                .last_mut()
                .expect("a player on a place")
                .outcome = outcome;
            self
        }

        fn gone(mut self) -> Self {
            self.scheduled.in_list = false;
            self
        }

        fn places(&self, today: NaiveDate) -> Option<GamePlaces> {
            let game = &self.scheduled.game;
            let filled: BTreeSet<(DutyKind, u8)> = self
                .assignments
                .iter()
                .map(|a| (a.kind, a.number))
                .collect();
            game_places(
                required_duties(game, CLUB, TEAM),
                Some(&self.stored),
                TEAM,
                &filled,
                9,
                self.scheduled.in_list,
                game.date < today,
            )
        }
    }

    fn message_with(
        fixtures: &[Fixture],
        names: &BTreeMap<PlayerId, String>,
        today: NaiveDate,
    ) -> Option<String> {
        let places: Vec<Option<GamePlaces>> = fixtures.iter().map(|f| f.places(today)).collect();
        let games: Vec<MessageGame<'_>> = fixtures
            .iter()
            .zip(&places)
            .map(|(fixture, places)| MessageGame {
                scheduled: &fixture.scheduled,
                places: places.as_ref(),
                assignments: &fixture.assignments,
            })
            .collect();
        week_message(&games, names, CLUB, TEAM, today)
    }

    /// The committee message of `fixtures`, with `names`.
    fn committee(
        fixtures: &[Fixture],
        names: &BTreeMap<PlayerId, String>,
        today: NaiveDate,
    ) -> String {
        let places: Vec<Option<GamePlaces>> = fixtures.iter().map(|f| f.places(today)).collect();
        let games: Vec<MessageGame<'_>> = fixtures
            .iter()
            .zip(&places)
            .map(|(fixture, places)| MessageGame {
                scheduled: &fixture.scheduled,
                places: places.as_ref(),
                assignments: &fixture.assignments,
            })
            .collect();
        committee_message(&games, names, "Blue Drakes", "M16 1", today)
    }

    fn full(entries: &[(PlayerId, &str)]) -> BTreeMap<PlayerId, String> {
        entries
            .iter()
            .map(|(id, name)| (*id, (*name).to_owned()))
            .collect()
    }

    const NO_DUTIES: &str = "Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november\n\
                             \n\
                             Geen tafel- of scheidsrechtersdiensten deze week.";

    fn message(fixtures: &[Fixture], today: NaiveDate) -> Option<String> {
        message_with(fixtures, &names(), today)
    }

    fn text(fixtures: &[Fixture], today: NaiveDate) -> String {
        message(fixtures, today).expect("a message")
    }

    /// The table and referee places of the example: table 1 Eva, table 2 open, referee 1 Gijs.
    fn example_duty() -> Fixture {
        Fixture::duty(2, date(2026, 10, 31), time(16, 0))
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2)
            .of_team(PlaceKind::Referee, 1)
            .player(DutyKind::Table, 1, EVA)
            .player(DutyKind::Referee, 1, GIJS)
    }

    /// A game with both table places of the team, open.
    fn example_duty_with_two_tables() -> Fixture {
        Fixture::duty(2, date(2026, 10, 31), time(16, 0))
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2)
    }

    #[test]
    fn the_week_runs_from_monday_to_sunday() {
        assert_eq!(week_of(monday()), (monday(), date(2026, 11, 1)));
        assert_eq!(week_of(date(2026, 11, 1)), (monday(), date(2026, 11, 1)));
        // The clock goes back on Sunday 25 October.
        assert_eq!(
            week_of(date(2026, 10, 25)),
            (date(2026, 10, 19), date(2026, 10, 25))
        );
        assert_eq!(
            week_of(date(2027, 1, 2)),
            (date(2026, 12, 28), date(2027, 1, 3))
        );
    }

    #[test]
    fn the_example_of_the_spec() {
        let away = Fixture::away(1, date(2026, 10, 31), time(14, 30))
            .drivers(4)
            .player(DutyKind::Drive, 1, ANNA)
            .player(DutyKind::Drive, 2, BRAM)
            .player(DutyKind::Drive, 3, CAS);
        assert_eq!(
            text(&[away, example_duty()], date(2026, 10, 28)),
            "Week 26 oktober – 1 november\n\
             \n\
             Wedstrijd: zaterdag 31 oktober, 14:30\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden\n\
             Rijden: Anna, Bram, Cas, nog 1 open\n\
             \n\
             Tafel en scheidsrechter:\n\
             za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Gijs\n  \
             Tafel: Eva, nog 1 open"
        );
    }

    #[test]
    fn the_heading_has_both_full_dates() {
        let in_october = Fixture::home(1, date(2026, 10, 17), time(11, 0));
        assert!(
            text(&[in_october], date(2026, 10, 12)).starts_with("Week 12 oktober – 18 oktober\n")
        );
        let over_new_year = Fixture::home(1, date(2027, 1, 2), time(11, 0));
        assert!(
            text(&[over_new_year], date(2026, 12, 28))
                .starts_with("Week 28 december – 3 januari\n")
        );
    }

    #[test]
    fn two_games_of_the_team_have_two_blocks_in_time_order() {
        let saturday = Fixture::away(1, date(2026, 10, 31), time(14, 30)).drivers(0);
        let wednesday = Fixture::home(3, date(2026, 10, 28), time(19, 0));
        assert_eq!(
            text(&[saturday, wednesday], monday()),
            "Week 26 oktober – 1 november\n\
             \n\
             Wedstrijd: woensdag 28 oktober, 19:00\n\
             Thuis tegen Lions M16 1\n\
             Sporthal Bulwijk, Woerden\n\
             \n\
             Wedstrijd: zaterdag 31 oktober, 14:30\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden"
        );
    }

    #[test]
    fn a_home_game_of_the_team_has_no_drivers() {
        let home = Fixture::home(1, date(2026, 10, 31), time(14, 30));
        assert_eq!(
            text(&[home], monday()),
            "Week 26 oktober – 1 november\n\
             \n\
             Wedstrijd: zaterdag 31 oktober, 14:30\n\
             Thuis tegen Lions M16 1\n\
             Sporthal Bulwijk, Woerden"
        );
    }

    #[test]
    fn a_game_against_another_club_team_in_the_clubs_hall_is_at_home() {
        // The team is the away side, but the game is in the club's own hall: no drivers.
        let game = Fixture::game(
            1,
            date(2026, 10, 31),
            time(14, 30),
            side(CLUB_TEAM, "Blue Drakes M16 2", CLUB),
            side(TEAM, "Blue Drakes M16 1", CLUB),
            ScheduleReason::TeamGame,
        );
        let text = text(&[game], monday());
        assert!(text.contains("\nThuis tegen Blue Drakes M16 2\n"));
        assert!(!text.contains("Rijden"));
    }

    #[test]
    fn a_start_time_not_known_yet() {
        let away = Fixture::away(1, date(2026, 10, 31), None).drivers(0);
        let duty = Fixture::duty(2, date(2026, 11, 1), None)
            .of_team(PlaceKind::Referee, 1)
            .player(DutyKind::Referee, 1, GIJS);
        assert_eq!(
            text(&[away, duty], monday()),
            "Week 26 oktober – 1 november\n\
             \n\
             Wedstrijd: zaterdag 31 oktober, tijd nog niet bekend\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden\n\
             \n\
             Tafel en scheidsrechter:\n\
             zo 1 nov, tijd nog niet bekend, Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Gijs"
        );
    }

    #[test]
    fn the_drivers_all_filled_none_filled_or_none_needed() {
        let on = date(2026, 10, 31);
        let all = Fixture::away(1, on, time(14, 30))
            .drivers(2)
            .player(DutyKind::Drive, 2, BRAM)
            .player(DutyKind::Drive, 1, ANNA);
        assert!(text(&[all], monday()).ends_with("Woerden\nRijden: Anna, Bram"));

        // Without a set number the proposal counts: 9 players need 3 drivers.
        let none = Fixture::away(1, on, time(14, 30));
        assert!(text(&[none], monday()).ends_with("Woerden\nRijden: nog 3 open"));

        let zero = Fixture::away(1, on, time(14, 30)).drivers(0);
        assert!(!text(&[zero], monday()).contains("Rijden"));
    }

    #[test]
    fn table_and_referee_all_filled_none_filled_and_one_kind() {
        let on = date(2026, 10, 31);
        let all = Fixture::duty(2, on, time(16, 0))
            .of_team(PlaceKind::Referee, 2)
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2)
            .player(DutyKind::Referee, 2, GIJS)
            .player(DutyKind::Table, 2, FLEUR)
            .player(DutyKind::Table, 1, EVA);
        assert!(
            text(&[all], monday()).ends_with("\n  Scheidsrechter: Gijs\n  Tafel: Eva, Fleur"),
            "Scheidsrechter comes before Tafel, the names in place order"
        );

        let none = Fixture::duty(2, on, time(16, 0))
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2);
        assert!(text(&[none], monday()).ends_with("\n  Tafel: nog 2 open"));

        let referee = Fixture::duty(2, on, time(16, 0))
            .of_team(PlaceKind::Referee, 1)
            .of_team(PlaceKind::Referee, 2)
            .player(DutyKind::Referee, 2, GIJS);
        let referee = text(&[referee], monday());
        assert!(referee.ends_with("\n  Scheidsrechter: Gijs, nog 1 open"));
        assert!(!referee.contains("Tafel:"));
    }

    #[test]
    fn places_of_other_club_teams_are_not_mentioned() {
        let game = Fixture::duty(2, date(2026, 10, 31), time(16, 0))
            .of_team(PlaceKind::Table, 1)
            .of_other_team(PlaceKind::Table, 2)
            .of_other_team(PlaceKind::Referee, 1)
            .player(DutyKind::Table, 1, EVA);
        let text = text(&[game], monday());
        assert!(text.ends_with("\n  Tafel: Eva"));
        assert!(!text.contains("Scheidsrechter:"));

        let others_only =
            Fixture::duty(2, date(2026, 10, 31), time(16, 0)).of_other_team(PlaceKind::Table, 1);
        assert_eq!(message(&[others_only], monday()), None);
    }

    #[test]
    fn a_free_week_has_the_heading_and_the_places() {
        assert_eq!(
            text(&[example_duty()], monday()),
            "Week 26 oktober – 1 november\n\
             \n\
             Tafel en scheidsrechter:\n\
             za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Gijs\n  \
             Tafel: Eva, nog 1 open"
        );
    }

    #[test]
    fn a_game_without_places_this_week_has_no_second_part() {
        let home = Fixture::home(1, date(2026, 10, 31), time(14, 30));
        let other = Fixture::duty(2, date(2026, 10, 31), time(16, 0));
        let text = text(&[home, other], monday());
        assert!(!text.contains("Tafel en scheidsrechter"));
        assert!(text.ends_with("Sporthal Bulwijk, Woerden"));
    }

    #[test]
    fn nothing_to_announce_gives_no_message() {
        assert_eq!(message(&[], monday()), None);
        let other = Fixture::duty(2, date(2026, 10, 31), time(16, 0));
        assert_eq!(message(&[other], monday()), None);

        // Only a game of the team that was played: nothing to announce.
        let played = Fixture::home(1, date(2026, 10, 27), time(19, 0));
        assert_eq!(message(&[played], date(2026, 10, 30)), None);
    }

    #[test]
    fn a_played_game_leaves_and_its_places_stay() {
        // Friday: the game of the team on Tuesday has no block; the places of Tuesday stay, in
        // date order, also when the games come in the order of the past games.
        let friday = date(2026, 10, 30);
        let played = Fixture::home(1, date(2026, 10, 27), time(19, 0));
        let duty = Fixture::duty(4, date(2026, 10, 27), time(20, 30))
            .of_team(PlaceKind::Referee, 1)
            .player(DutyKind::Referee, 1, GIJS);
        assert_eq!(
            text(&[example_duty(), played, duty], friday),
            "Week 26 oktober – 1 november\n\
             \n\
             Tafel en scheidsrechter:\n\
             di 27 okt 20:30 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Gijs\n\
             \n\
             za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Gijs\n  \
             Tafel: Eva, nog 1 open"
        );
    }

    #[test]
    fn a_game_today_stays_all_day() {
        // The clock is not read: a game today at 10:00 stays in the evening.
        let today = Fixture::home(1, monday(), time(10, 0));
        assert!(text(&[today], monday()).contains("Wedstrijd: maandag 26 oktober, 10:00"));
    }

    #[test]
    fn a_past_place_shows_who_did_it() {
        let tuesday = date(2026, 10, 27);
        let game = Fixture::duty(2, tuesday, time(19, 0))
            .of_team(PlaceKind::Referee, 1)
            .of_team(PlaceKind::Referee, 2)
            .of_team(PlaceKind::Table, 1)
            .player(DutyKind::Referee, 1, GIJS)
            .player(DutyKind::Referee, 2, EVA)
            .outcome(Outcome::NotDone)
            .player(DutyKind::Table, 1, ANNA)
            .outcome(Outcome::DoneBy(BRAM));
        assert!(
            text(&[game], date(2026, 10, 30))
                .ends_with("\n  Scheidsrechter: Gijs, niemand\n  Tafel: Bram")
        );

        // Before the game the planned player shows, whatever is stored.
        let coming = Fixture::duty(2, tuesday, time(19, 0))
            .of_team(PlaceKind::Table, 1)
            .player(DutyKind::Table, 1, ANNA)
            .outcome(Outcome::DoneBy(BRAM));
        assert!(text(&[coming], monday()).ends_with("\n  Tafel: Anna"));
    }

    #[test]
    fn a_past_place_that_stayed_open_shows_niemand() {
        let friday = date(2026, 10, 30);
        let tuesday = date(2026, 10, 27);
        let one_open = Fixture::duty(2, tuesday, time(19, 0))
            .of_team(PlaceKind::Referee, 1)
            .of_team(PlaceKind::Referee, 2)
            .player(DutyKind::Referee, 2, GIJS);
        assert!(text(&[one_open], friday).ends_with("\n  Scheidsrechter: niemand, Gijs"));

        // With something else in the week it stays in the message.
        let open = Fixture::duty(2, tuesday, time(19, 0)).of_team(PlaceKind::Table, 1);
        let text = text(&[open, example_duty()], friday);
        assert!(text.contains(
            "di 27 okt 19:00 Blue Drakes X12 1 – De Dunckers X12 1\n  Tafel: niemand\n\n"
        ));

        // Alone it announces nothing.
        let open = Fixture::duty(2, tuesday, time(19, 0)).of_team(PlaceKind::Table, 1);
        assert_eq!(message(&[open], friday), None);
    }

    #[test]
    fn on_a_sunday_the_whole_week_is_in_it_and_not_the_next_monday() {
        let sunday = date(2026, 11, 1);
        let earlier = Fixture::duty(2, monday(), time(19, 0))
            .of_team(PlaceKind::Table, 1)
            .player(DutyKind::Table, 1, EVA);
        let next_monday =
            Fixture::duty(4, date(2026, 11, 2), time(19, 0)).of_team(PlaceKind::Referee, 1);
        let text = text(&[earlier, next_monday], sunday);
        assert!(text.contains("ma 26 okt 19:00"));
        assert!(!text.contains("ma 2 nov"));
    }

    #[test]
    fn places_that_do_not_count_are_left_out() {
        // A coming game no longer in the game list.
        let gone = example_duty().gone();
        assert_eq!(message(&[gone], monday()), None);

        // A past game that left the game list keeps what is stored.
        let gone_past = example_duty().gone();
        assert!(text(&[gone_past], date(2026, 11, 1)).contains("za 31 okt 16:00"));

        // A game no longer a home game of the club (home and away swapped): its places stay
        // stored but do not count, and the game is kept only for its duties.
        let mut swapped = example_duty();
        swapped.scheduled.game.home = side(TeamNumber(901), "De Dunckers X12 1", OTHER_CLUB);
        swapped.scheduled.game.away = side(CLUB_TEAM, "Blue Drakes X12 1", CLUB);
        swapped.scheduled.reason = ScheduleReason::Kept;
        assert_eq!(message(&[swapped], monday()), None);
    }

    #[test]
    fn a_game_of_the_team_needs_to_be_in_the_game_list() {
        let gone = Fixture::away(1, date(2026, 10, 31), time(14, 30)).gone();
        assert_eq!(message(&[gone], monday()), None);

        let mut kept = Fixture::away(1, date(2026, 10, 31), time(14, 30));
        kept.scheduled.reason = ScheduleReason::Kept;
        assert_eq!(message(&[kept], monday()), None);
    }

    #[test]
    fn a_game_after_this_week_is_not_in_it() {
        let later = Fixture::away(1, date(2026, 11, 7), time(12, 0));
        assert_eq!(message(&[later], monday()), None);
    }

    #[test]
    fn the_committee_example_of_the_spec() {
        let sunday = Fixture::game(
            4,
            date(2026, 11, 1),
            time(11, 0),
            side(CLUB_TEAM, "Blue Drakes M14 2", CLUB),
            side(TeamNumber(902), "Lions M14 1", OTHER_CLUB),
            ScheduleReason::DutyGame,
        )
        .of_team(PlaceKind::Referee, 1)
        .of_team(PlaceKind::Table, 1)
        .player(DutyKind::Table, 1, ANNA);
        let saturday = Fixture::duty(2, date(2026, 10, 31), time(13, 0))
            .of_team(PlaceKind::Referee, 1)
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2)
            .player(DutyKind::Referee, 1, EVA)
            .player(DutyKind::Table, 1, FLEUR)
            .player(DutyKind::Table, 2, GIJS);
        let names = full(&[
            (ANNA, "Hugo Smit"),
            (EVA, "Eva de Vries"),
            (FLEUR, "Fleur Jansen"),
            (GIJS, "Gijs Bakker"),
        ]);
        assert_eq!(
            committee(&[sunday, saturday], &names, monday()),
            "Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november\n\
             \n\
             za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Scheidsrechter: Eva de Vries\n  \
             Tafel: Fleur Jansen, Gijs Bakker\n\
             \n\
             zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1\n  \
             Scheidsrechter: nog 1 open\n  \
             Tafel: Hugo Smit"
        );
    }

    #[test]
    fn a_week_without_places_says_so_to_the_committee() {
        let names = full(&[]);
        assert_eq!(committee(&[], &names, monday()), NO_DUTIES);

        // Places of other club teams and a game of the team without places do not count.
        let others =
            Fixture::duty(2, date(2026, 10, 31), time(16, 0)).of_other_team(PlaceKind::Table, 1);
        let game = Fixture::away(1, date(2026, 10, 31), time(14, 30));
        assert_eq!(committee(&[others, game], &names, monday()), NO_DUTIES);
    }

    #[test]
    fn full_names_have_the_first_and_the_last_name() {
        let player = |id, first: &str, last: Option<&str>| Player {
            id: PlayerId(id),
            first_name: first.to_owned(),
            last_name: last.map(str::to_owned),
            hidden: false,
        };
        let names = full_names(&[player(1, "Eva", Some("de Vries")), player(2, "Hugo", None)]);
        assert_eq!(names[&PlayerId(1)], "Eva de Vries");
        assert_eq!(names[&PlayerId(2)], "Hugo");
    }

    #[test]
    fn the_committee_message_shows_open_places() {
        let names = full(&[(EVA, "Eva de Vries")]);
        let game = example_duty_with_two_tables().player(DutyKind::Table, 1, EVA);
        assert!(
            committee(&[game], &names, monday()).ends_with("\n  Tafel: Eva de Vries, nog 1 open")
        );
        let open = example_duty_with_two_tables();
        assert!(committee(&[open], &names, monday()).ends_with("\n  Tafel: nog 2 open"));
    }

    #[test]
    fn the_committee_message_shows_who_did_a_past_place() {
        let friday = date(2026, 10, 30);
        let tuesday = date(2026, 10, 27);
        let names = full(&[
            (ANNA, "Anna Bakker"),
            (BRAM, "Bram Smit"),
            (EVA, "Eva de Vries"),
        ]);
        let game = Fixture::duty(2, tuesday, time(19, 0))
            .of_team(PlaceKind::Referee, 1)
            .of_team(PlaceKind::Referee, 2)
            .of_team(PlaceKind::Table, 1)
            .player(DutyKind::Referee, 1, ANNA)
            .outcome(Outcome::DoneBy(BRAM))
            .player(DutyKind::Table, 1, EVA)
            .outcome(Outcome::NotDone);
        assert!(
            committee(&[game], &names, friday)
                .ends_with("\n  Scheidsrechter: Bram Smit, niemand\n  Tafel: niemand")
        );

        // A past place that stayed open, alone: the team had a place, so not "Geen ...".
        let open = Fixture::duty(2, tuesday, time(19, 0)).of_team(PlaceKind::Table, 1);
        assert!(committee(&[open], &names, friday).ends_with("\n  Tafel: niemand"));
    }

    #[test]
    fn the_committee_message_follows_what_counts_and_the_week() {
        let names = full(&[(EVA, "Eva"), (GIJS, "Gijs")]);
        // A coming game no longer in the game list is left out.
        assert_eq!(
            committee(&[example_duty().gone()], &names, monday()),
            NO_DUTIES
        );
        // A coming game no longer a home game of the club is left out.
        let mut swapped = example_duty();
        swapped.scheduled.game.home = side(TeamNumber(901), "De Dunckers X12 1", OTHER_CLUB);
        swapped.scheduled.game.away = side(CLUB_TEAM, "Blue Drakes X12 1", CLUB);
        swapped.scheduled.reason = ScheduleReason::Kept;
        assert_eq!(committee(&[swapped], &names, monday()), NO_DUTIES);
        // A past game that left the game list stays.
        assert!(
            committee(&[example_duty().gone()], &names, date(2026, 11, 1))
                .contains("za 31 okt 16:00")
        );
        // On a Sunday the whole week is in it, and not the next Monday.
        let earlier = Fixture::duty(2, monday(), time(19, 0)).of_team(PlaceKind::Table, 1);
        let next = Fixture::duty(4, date(2026, 11, 2), time(19, 0)).of_team(PlaceKind::Table, 1);
        let text = committee(&[earlier, next], &names, date(2026, 11, 1));
        assert!(text.contains("ma 26 okt 19:00"));
        assert!(!text.contains("ma 2 nov"));
    }

    #[test]
    fn two_players_with_the_same_first_name_show_the_first_letter_of_the_last_name() {
        let anna = |id, last: &str| Player {
            id: PlayerId(id),
            first_name: "Anna".to_owned(),
            last_name: Some(last.to_owned()),
            hidden: false,
        };
        let names = display_names(&[anna(1, "Bakker"), anna(2, "Visser")]);
        let away = Fixture::away(1, date(2026, 10, 31), time(14, 30))
            .drivers(2)
            .player(DutyKind::Drive, 1, PlayerId(1))
            .player(DutyKind::Drive, 2, PlayerId(2));
        let text = message_with(&[away], &names, monday()).expect("a message");
        assert!(text.ends_with("Rijden: Anna B., Anna V."));
    }
}
