//! Strict parsing of the source's answers, field by field, so that an error names the field.
//! Fields the tool does not use (scores, results, logos, counts) are not read.

use chrono::{NaiveDate, NaiveTime};
use serde_json::{Map, Value};
use team_core::{
    Club, ClubNumber, Game, Place, PlaceNumber, Season, Side, TeamNumber, start_time_from_source,
};

use super::SourceError;

type Object = Map<String, Value>;

fn format_error(place: impl Into<String>, reason: impl Into<String>) -> SourceError {
    SourceError::Format {
        place: place.into(),
        reason: reason.into(),
    }
}

fn root(body: &[u8]) -> Result<Object, SourceError> {
    match serde_json::from_slice(body) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(format_error("answer", "expected a JSON object")),
        Err(error) => Err(format_error("answer", error.to_string())),
    }
}

fn list<'a>(object: &'a Object, field: &str) -> Result<&'a Vec<Value>, SourceError> {
    match object.get(field) {
        Some(Value::Array(items)) => Ok(items),
        Some(_) => Err(format_error(format!("field {field}"), "expected a list")),
        None => Err(format_error(format!("field {field}"), "missing")),
    }
}

/// Reads the fields of one item and names the item in each error.
struct Fields<'a> {
    object: &'a Object,
    item: String,
}

impl<'a> Fields<'a> {
    fn new(value: &'a Value, kind: &str, position: usize) -> Result<Self, SourceError> {
        let Value::Object(object) = value else {
            return Err(format_error(
                format!("{kind} {} in the list", position + 1),
                "expected an object",
            ));
        };
        // Name the item by its id when it has one, else by its position.
        let item = match object.get("id").and_then(Value::as_i64) {
            Some(id) => format!("{kind} {id}"),
            None => format!("{kind} {} in the list", position + 1),
        };
        Ok(Self { object, item })
    }

    fn error(&self, field: &str, reason: impl Into<String>) -> SourceError {
        format_error(format!("{}, field {field}", self.item), reason)
    }

    fn get(&self, field: &str) -> Result<&'a Value, SourceError> {
        self.object
            .get(field)
            .ok_or_else(|| self.error(field, "missing"))
    }

    fn number(&self, field: &str) -> Result<i64, SourceError> {
        self.get(field)?
            .as_i64()
            .ok_or_else(|| self.error(field, "expected a whole number"))
    }

    fn text(&self, field: &str) -> Result<&'a str, SourceError> {
        self.get(field)?
            .as_str()
            .ok_or_else(|| self.error(field, "expected a text"))
    }

    fn filled_text(&self, field: &str) -> Result<String, SourceError> {
        let text = self.text(field)?.trim();
        if text.is_empty() {
            return Err(self.error(field, "is empty"));
        }
        Ok(text.to_owned())
    }

    /// A text that may be empty or `null`; both give `None`.
    fn optional_text(&self, field: &str) -> Result<Option<String>, SourceError> {
        match self.get(field)? {
            Value::Null => Ok(None),
            Value::String(text) if text.trim().is_empty() => Ok(None),
            Value::String(text) => Ok(Some(text.trim().to_owned())),
            _ => Err(self.error(field, "expected a text or null")),
        }
    }
}

pub(super) fn clubs(body: &[u8]) -> Result<Vec<Club>, SourceError> {
    let root = root(body)?;
    list(&root, "clubs")?
        .iter()
        .enumerate()
        .map(|(position, value)| {
            let fields = Fields::new(value, "club", position)?;
            Ok(Club {
                number: ClubNumber(fields.number("id")?),
                name: fields.filled_text("naam")?,
                town: fields.optional_text("plaats")?,
            })
        })
        .collect()
}

pub(super) fn games(body: &[u8], season: Season) -> Result<Vec<Game>, SourceError> {
    let root = root(body)?;
    list(&root, "wedstrijden")?
        .iter()
        .enumerate()
        .map(|(position, value)| game(&Fields::new(value, "game", position)?, season))
        .collect()
}

fn game(fields: &Fields<'_>, season: Season) -> Result<Game, SourceError> {
    let game_season: Season = fields
        .text("seizoen")?
        .parse()
        .map_err(|_| fields.error("seizoen", "expected a season such as \"2026-2027\""))?;
    if game_season != season {
        return Err(fields.error("seizoen", format!("is {game_season}, asked for {season}")));
    }
    Ok(Game {
        source_id: fields.number("id")?,
        game_number: fields.filled_text("nr")?,
        season,
        date: date(fields)?,
        start_time: start_time_from_source(time(fields)?),
        home: Side {
            team: TeamNumber(fields.number("thuis_ploeg_id")?),
            team_name: fields.filled_text("thuis_ploeg")?,
            club: ClubNumber(fields.number("thuis_club_id")?),
        },
        away: Side {
            team: TeamNumber(fields.number("uit_ploeg_id")?),
            team_name: fields.filled_text("uit_ploeg")?,
            club: ClubNumber(fields.number("uit_club_id")?),
        },
        place: Place {
            number: PlaceNumber(fields.number("loc_id")?),
            hall: fields.filled_text("loc_naam")?,
            town: fields.filled_text("loc_plaats")?,
            court: fields.optional_text("veld")?,
        },
        competition: fields.filled_text("cmp_naam")?,
    })
}

/// The date part of `datum`, for example "2026-09-12" of "2026-09-12 11:15:00.000".
fn date(fields: &Fields<'_>) -> Result<NaiveDate, SourceError> {
    fields
        .text("datum")?
        .get(..10)
        .and_then(|part| NaiveDate::parse_from_str(part, "%Y-%m-%d").ok())
        .ok_or_else(|| fields.error("datum", "expected a date such as \"2026-09-12 11:15\""))
}

/// `tijd` as "HH:MM", Dutch local time.
fn time(fields: &Fields<'_>) -> Result<NaiveTime, SourceError> {
    let text = fields.text("tijd")?;
    let parsed = (text.len() == 5)
        .then(|| NaiveTime::parse_from_str(text, "%H:%M").ok())
        .flatten();
    parsed.ok_or_else(|| fields.error("tijd", "expected a time such as \"11:15\""))
}
