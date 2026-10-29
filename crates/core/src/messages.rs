//! The messages for the coordinator to copy into the chat (story C of duties): Dutch text, made
//! from what is stored at the moment of the request. The tool never sends them.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate, NaiveTime};

use crate::club::ClubNumber;
use crate::duties::{Assignment, DutyKind, DutyPlace, GamePlaces};
use crate::game::{Game, TeamNumber};
use crate::players::PlayerId;
use crate::schedule::{ScheduleReason, ScheduledGame};

/// A coming game of the season with its duties, as the messages read it.
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

/// The last day of the coming week: the coming Sunday; today on a Sunday.
pub fn coming_week(today: NaiveDate) -> NaiveDate {
    let days_left = 6 - u64::from(today.weekday().num_days_from_monday());
    today
        .checked_add_days(Days::new(days_left))
        .unwrap_or(today)
}

/// "zaterdag 31 oktober".
fn long_date(date: NaiveDate) -> String {
    let day = DAYS[date.weekday().num_days_from_monday() as usize];
    let month = MONTHS[date.month0() as usize];
    format!("{day} {} {month}", date.day())
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
/// "nog 2 open". `brackets`: the open places in brackets ("Anna (nog 1 open)"), as for drivers.
fn filled_and_open(
    places: &[&DutyPlace],
    assignments: &[Assignment],
    names: &BTreeMap<PlayerId, String>,
    brackets: bool,
) -> String {
    let filled: Vec<&str> = places
        .iter()
        .filter(|place| place.filled)
        .map(|place| {
            assignments
                .iter()
                .find(|a| a.kind == place.kind && a.number == place.number)
                .and_then(|a| names.get(&a.planned))
                .map_or("?", String::as_str)
        })
        .collect();
    let open = places.iter().filter(|place| !place.filled).count();
    let names = filled.join(", ");
    match (filled.is_empty(), open) {
        (_, 0) => names,
        (true, open) => format!("nog {open} open"),
        (false, open) if brackets => format!("{names} (nog {open} open)"),
        (false, open) => format!("{names}, nog {open} open"),
    }
}

/// The part about the team's next game, with its drivers. "Thuis": a game in the club's own hall
/// (`docs/language.md`), also with the team as the away side against another club team.
fn next_game_part(
    next: &MessageGame<'_>,
    names: &BTreeMap<PlayerId, String>,
    club: ClubNumber,
    team: TeamNumber,
) -> Vec<String> {
    let game = &next.scheduled.game;
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
    let mut parts = vec![format!(
        "Volgende wedstrijd: {}, {time}\n{side} tegen {opponent}\n{}, {}",
        long_date(game.date),
        game.place.hall,
        game.place.town
    )];
    let drivers = next
        .places
        .and_then(|places| places.drivers.as_ref())
        .filter(|part| part.counts && !part.places.is_empty());
    if let Some(part) = drivers {
        let places: Vec<&DutyPlace> = part.places.iter().collect();
        parts.push(format!(
            "Rijden: {}",
            filled_and_open(&places, next.assignments, names, true)
        ));
    }
    parts
}

/// The lines of one game of the week part; `None`: no table or referee places of the team that
/// count.
fn week_game(game: &MessageGame<'_>, names: &BTreeMap<PlayerId, String>) -> Option<String> {
    let part = game
        .places
        .and_then(|places| places.schedule.as_ref())
        .filter(|part| part.counts && !part.of_team.is_empty())?;
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
        (DutyKind::Table, "Tafel"),
        (DutyKind::Referee, "Scheidsrechter"),
    ] {
        let places: Vec<&DutyPlace> = part.of_team.iter().filter(|p| p.kind == kind).collect();
        if !places.is_empty() {
            lines.push(format!(
                "  {label}: {}",
                filled_and_open(&places, game.assignments, names, false)
            ));
        }
    }
    Some(lines.join("\n"))
}

/// The message for the next game. `games`: the coming games in time order; `names`:
/// `display_names` of the season; `club` and `team`: the club and the team chosen for the season.
/// The next game is the first game of the team still in the game
/// list (a game kept only for its duties never is); the week part has the games from today to
/// `coming_week(today)` with table or referee places of the team that count. `None`: neither.
pub fn next_game_message(
    games: &[MessageGame<'_>],
    names: &BTreeMap<PlayerId, String>,
    club: ClubNumber,
    team: TeamNumber,
    today: NaiveDate,
) -> Option<String> {
    let until = coming_week(today);
    let next = games.iter().find(|game| {
        game.scheduled.reason == ScheduleReason::TeamGame
            && game.scheduled.in_list
            && game.scheduled.game.date >= today
    });
    let week: Vec<String> = games
        .iter()
        .filter(|game| (today..=until).contains(&game.scheduled.game.date))
        .filter_map(|game| week_game(game, names))
        .collect();

    let mut parts = next.map_or_else(Vec::new, |next| next_game_part(next, names, club, team));
    if !week.is_empty() {
        parts.push(format!(
            "Tafel en scheidsrechter deze week:\n{}",
            week.join("\n")
        ));
    }
    (!parts.is_empty()).then(|| parts.join("\n\n"))
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
        next_game_message(&games, names, CLUB, TEAM, today)
    }

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

    #[test]
    fn the_coming_week_ends_on_the_coming_sunday() {
        assert_eq!(coming_week(monday()), date(2026, 11, 1));
        assert_eq!(coming_week(date(2026, 10, 24)), date(2026, 10, 25));
        // The clock goes back on Sunday 25 October: on a Sunday the week is that day only.
        assert_eq!(coming_week(date(2026, 10, 25)), date(2026, 10, 25));
        assert_eq!(coming_week(date(2026, 12, 27)), date(2026, 12, 27));
        assert_eq!(coming_week(date(2026, 12, 28)), date(2027, 1, 3));
    }

    #[test]
    fn the_example_of_the_spec() {
        let away = Fixture::away(1, date(2026, 10, 31), time(14, 30))
            .drivers(4)
            .player(DutyKind::Drive, 1, ANNA)
            .player(DutyKind::Drive, 2, BRAM)
            .player(DutyKind::Drive, 3, CAS);
        assert_eq!(
            text(&[away, example_duty()], monday()),
            "Volgende wedstrijd: zaterdag 31 oktober, 14:30\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden\n\
             \n\
             Rijden: Anna, Bram, Cas (nog 1 open)\n\
             \n\
             Tafel en scheidsrechter deze week:\n\
             za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Tafel: Eva, nog 1 open\n  \
             Scheidsrechter: Gijs"
        );
    }

    #[test]
    fn a_home_game_of_the_team_has_no_drivers() {
        let home = Fixture::home(1, date(2026, 10, 31), time(14, 30));
        assert_eq!(
            text(&[home], monday()),
            "Volgende wedstrijd: zaterdag 31 oktober, 14:30\n\
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
        assert_eq!(
            text(&[game], monday()),
            "Volgende wedstrijd: zaterdag 31 oktober, 14:30\n\
             Thuis tegen Blue Drakes M16 2\n\
             Sporthal Bulwijk, Woerden"
        );
    }

    #[test]
    fn a_start_time_not_known_yet() {
        let away = Fixture::away(1, date(2026, 10, 31), None).drivers(0);
        let duty = Fixture::duty(2, date(2026, 11, 1), None)
            .of_team(PlaceKind::Referee, 1)
            .player(DutyKind::Referee, 1, GIJS);
        assert_eq!(
            text(&[away, duty], monday()),
            "Volgende wedstrijd: zaterdag 31 oktober, tijd nog niet bekend\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden\n\
             \n\
             Tafel en scheidsrechter deze week:\n\
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
        assert!(text(&[all], monday()).ends_with("\n\nRijden: Anna, Bram"));

        // Without a set number the proposal counts: 9 players need 3 drivers.
        let none = Fixture::away(1, on, time(14, 30));
        assert!(text(&[none], monday()).ends_with("\n\nRijden: nog 3 open"));

        let zero = Fixture::away(1, on, time(14, 30)).drivers(0);
        assert!(!text(&[zero], monday()).contains("Rijden"));
    }

    #[test]
    fn table_and_referee_all_filled_none_filled_and_only_referee() {
        let on = date(2026, 10, 31);
        let all = Fixture::duty(2, on, time(16, 0))
            .of_team(PlaceKind::Referee, 2)
            .of_team(PlaceKind::Table, 1)
            .of_team(PlaceKind::Table, 2)
            .player(DutyKind::Referee, 2, GIJS)
            .player(DutyKind::Table, 2, FLEUR)
            .player(DutyKind::Table, 1, EVA);
        assert!(
            text(&[all], monday()).ends_with("\n  Tafel: Eva, Fleur\n  Scheidsrechter: Gijs"),
            "Tafel comes before Scheidsrechter, the names in place order"
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
    fn without_a_next_game_only_the_week_part() {
        assert_eq!(
            text(&[example_duty()], monday()),
            "Tafel en scheidsrechter deze week:\n\
             za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
             Tafel: Eva, nog 1 open\n  \
             Scheidsrechter: Gijs"
        );
    }

    #[test]
    fn a_next_game_without_places_this_week_has_no_week_part() {
        let home = Fixture::home(1, date(2026, 10, 31), time(14, 30));
        let other = Fixture::duty(2, date(2026, 10, 31), time(16, 0));
        let text = text(&[home, other], monday());
        assert!(!text.contains("deze week"));
        assert!(text.ends_with("Sporthal Bulwijk, Woerden"));
    }

    #[test]
    fn nothing_coming_gives_no_message() {
        assert_eq!(message(&[], monday()), None);
        let other = Fixture::duty(2, date(2026, 10, 31), time(16, 0));
        assert_eq!(message(&[other], monday()), None);
    }

    #[test]
    fn places_that_do_not_count_are_left_out() {
        // A coming game no longer in the game list.
        let gone = example_duty().gone();
        assert_eq!(message(&[gone], monday()), None);

        // A game no longer a home game of the club (home and away swapped): its places stay
        // stored but do not count, and the game is kept only for its duties.
        let mut swapped = example_duty();
        swapped.scheduled.game.home = side(TeamNumber(901), "De Dunckers X12 1", OTHER_CLUB);
        swapped.scheduled.game.away = side(CLUB_TEAM, "Blue Drakes X12 1", CLUB);
        swapped.scheduled.reason = ScheduleReason::Kept;
        assert_eq!(message(&[swapped], monday()), None);

        // A game of the team kept only for its duties is never the next game.
        let mut kept = Fixture::away(1, date(2026, 10, 31), time(14, 30));
        kept.scheduled.reason = ScheduleReason::Kept;
        let next = Fixture::home(3, date(2026, 11, 7), time(11, 0));
        assert!(text(&[kept, next], monday()).contains("zaterdag 7 november"));
    }

    #[test]
    fn a_next_game_no_longer_in_the_game_list_gives_way_to_the_next_one_that_is() {
        let gone = Fixture::away(1, date(2026, 10, 31), time(14, 30)).gone();
        let next = Fixture::home(3, date(2026, 11, 7), time(11, 0));
        assert_eq!(
            text(&[gone, next], monday()),
            "Volgende wedstrijd: zaterdag 7 november, 11:00\n\
             Thuis tegen Lions M16 1\n\
             Sporthal Bulwijk, Woerden"
        );
    }

    #[test]
    fn the_week_part_ends_on_the_coming_sunday() {
        let monday_after =
            Fixture::duty(2, date(2026, 11, 2), time(19, 0)).of_team(PlaceKind::Table, 1);
        assert_eq!(message(&[monday_after], monday()), None);

        // On a Sunday only that day.
        let sunday = date(2026, 11, 1);
        let today = Fixture::duty(2, sunday, time(10, 0)).of_team(PlaceKind::Table, 1);
        let tomorrow =
            Fixture::duty(4, date(2026, 11, 2), time(19, 0)).of_team(PlaceKind::Referee, 1);
        let text = text(&[today, tomorrow], sunday);
        assert!(text.contains("zo 1 nov 10:00"));
        assert!(!text.contains("ma 2 nov"));
    }

    #[test]
    fn a_game_today_is_the_next_game_all_day() {
        // The clock is not read: a game today at 10:00 stays the next game in the evening.
        let today = Fixture::home(1, monday(), time(10, 0));
        let later = Fixture::home(3, date(2026, 10, 31), time(14, 30));
        assert!(
            text(&[today, later], monday())
                .starts_with("Volgende wedstrijd: maandag 26 oktober, 10:00")
        );
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

    #[test]
    fn the_next_game_weeks_away_is_still_the_next_game() {
        let later = Fixture::away(1, date(2026, 11, 21), time(12, 0)).drivers(0);
        assert_eq!(
            text(&[later], monday()),
            "Volgende wedstrijd: zaterdag 21 november, 12:00\n\
             Uit tegen B.Z. '72 M16 1\n\
             Sporthal Bulwijk, Woerden"
        );
    }
}
