//! Parseur et calculateur cron à cinq champs, sans dépendance externe.
//!
//! Les occurrences sont évaluées en UTC à la minute. Les champs acceptent les nombres,
//! listes, plages et pas (par exemple \`*/15 9-17 * * 1-5\`). Les noms de mois/jours,
//! les macros et les extensions spécifiques à certains dialectes sont volontairement exclus.

use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use thiserror::Error;

const SEARCH_LIMIT_MINUTES: usize = 366 * 5 * 24 * 60;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CronError {
    #[error("Une expression cron doit contenir exactement cinq champs : minute, heure, jour du mois, mois, jour de semaine.")]
    FieldCount,
    #[error("Champ cron invalide « {field} » ({value:?}) : {reason}")]
    InvalidField {
        field: &'static str,
        value: String,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    min: u32,
    allowed: Vec<bool>,
}

impl Field {
    fn contains(&self, value: u32) -> bool {
        value
            .checked_sub(self.min)
            .and_then(|index| self.allowed.get(index as usize))
            .copied()
            .unwrap_or(false)
    }
}

/// Expression cron numérique à cinq champs. Cette première version utilise l'UTC ;
/// l'application devra enregistrer explicitement un fuseau avant de proposer l'heure locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronSchedule {
    expression: String,
    minute: Field,
    hour: Field,
    day_of_month: Field,
    month: Field,
    day_of_week: Field,
    day_of_month_unrestricted: bool,
    day_of_week_unrestricted: bool,
}

impl CronSchedule {
    pub fn parse(expression: &str) -> Result<Self, CronError> {
        let fields: Vec<&str> = expression.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(CronError::FieldCount);
        }

        Ok(Self {
            expression: expression.trim().to_string(),
            minute: parse_field(fields[0], 0, 59, false, "minute")?,
            hour: parse_field(fields[1], 0, 23, false, "heure")?,
            day_of_month: parse_field(fields[2], 1, 31, false, "jour du mois")?,
            month: parse_field(fields[3], 1, 12, false, "mois")?,
            day_of_week: parse_field(fields[4], 0, 7, true, "jour de semaine")?,
            day_of_month_unrestricted: fields[2] == "*",
            day_of_week_unrestricted: fields[4] == "*",
        })
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Indique si l'expression correspond exactement à cette minute UTC.
    pub fn matches_at(&self, instant: &DateTime<Utc>) -> bool {
        if instant.second() != 0 || instant.nanosecond() != 0 {
            return false;
        }

        let minute_matches = self.minute.contains(instant.minute());
        let hour_matches = self.hour.contains(instant.hour());
        let month_matches = self.month.contains(instant.month());
        let day_of_month_matches = self.day_of_month.contains(instant.day());
        let day_of_week_matches = self
            .day_of_week
            .contains(instant.weekday().num_days_from_sunday());

        let calendar_day_matches = match (
            self.day_of_month_unrestricted,
            self.day_of_week_unrestricted,
        ) {
            (true, true) => true,
            (true, false) => day_of_week_matches,
            (false, true) => day_of_month_matches,
            // Comportement cron classique : si les deux champs sont restreints,
            // une correspondance de l'un ou de l'autre suffit.
            (false, false) => day_of_month_matches || day_of_week_matches,
        };

        minute_matches && hour_matches && month_matches && calendar_day_matches
    }

    /// Retourne la prochaine occurrence strictement après l'instant donné.
    /// La recherche est bornée à cinq ans pour éviter une boucle infinie sur une
    /// combinaison valide syntaxiquement mais impossible (par exemple le 31 février).
    pub fn next_after(&self, after: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        let rounded = after
            .with_second(0)?
            .with_nanosecond(0)?
            .checked_add_signed(Duration::minutes(1))?;
        let mut cursor = rounded;

        for _ in 0..SEARCH_LIMIT_MINUTES {
            if self.matches_at(&cursor) {
                return Some(cursor);
            }
            cursor = cursor.checked_add_signed(Duration::minutes(1))?;
        }
        None
    }
}

fn parse_field(
    input: &str,
    min: u32,
    max: u32,
    sunday_seven_alias: bool,
    name: &'static str,
) -> Result<Field, CronError> {
    let invalid = |reason| CronError::InvalidField {
        field: name,
        value: input.to_string(),
        reason,
    };

    if input.is_empty() {
        return Err(invalid("champ vide"));
    }

    // Le jour de semaine accepte 0–7 mais 7 est un alias de dimanche (0).
    let mask_len = if sunday_seven_alias {
        (max - min) as usize + 1
    } else {
        (max - min + 1) as usize
    };
    let mut allowed = vec![false; mask_len];

    for item in input.split(',') {
        if item.is_empty() {
            return Err(invalid("élément vide dans la liste"));
        }

        let (range, step) = match item.split_once('/') {
            Some((range, step)) => {
                if range.is_empty() || step.is_empty() || step.contains('/') {
                    return Err(invalid("pas invalide"));
                }
                let step = step
                    .parse::<u32>()
                    .map_err(|_| invalid("le pas doit être un entier positif"))?;
                if step == 0 {
                    return Err(invalid("le pas doit être strictement positif"));
                }
                (range, Some(step))
            }
            None => (item, None),
        };

        let (start, end) = if range == "*" {
            (min, max)
        } else if let Some((start, end)) = range.split_once('-') {
            if end.contains('-') {
                return Err(invalid("plage invalide"));
            }
            let start = parse_number(start).ok_or_else(|| invalid("début de plage invalide"))?;
            let end = parse_number(end).ok_or_else(|| invalid("fin de plage invalide"))?;
            (start, end)
        } else {
            let start = parse_number(range).ok_or_else(|| invalid("valeur non numérique"))?;
            (start, if step.is_some() { max } else { start })
        };

        if start < min || start > max || end < min || end > max {
            return Err(invalid("valeur hors limites"));
        }
        if start > end {
            return Err(invalid("début de plage supérieur à la fin"));
        }

        let step = step.unwrap_or(1) as usize;
        for value in (start..=end).step_by(step) {
            let normalized = if sunday_seven_alias && value == 7 {
                0
            } else {
                value
            };
            let index = (normalized - min) as usize;
            if let Some(slot) = allowed.get_mut(index) {
                *slot = true;
            }
        }
    }

    if !allowed.iter().any(|value| *value) {
        return Err(invalid("le champ ne sélectionne aucune valeur"));
    }
    Ok(Field { min, allowed })
}

fn parse_number(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("date de test valide")
            .with_timezone(&Utc)
    }

    #[test]
    fn accepts_lists_ranges_steps_and_wildcards() {
        let schedule = CronSchedule::parse("*/15 9-10 1,15 1,6 1-5").unwrap();
        assert_eq!(schedule.expression(), "*/15 9-10 1,15 1,6 1-5");
        assert!(schedule.matches_at(&utc("2026-06-15T09:30:00Z")));
        assert!(!schedule.matches_at(&utc("2026-06-15T09:31:00Z")));
        assert!(!schedule.matches_at(&utc("2026-06-14T09:30:00Z")));
        assert!(!schedule.matches_at(&utc("2026-10-15T09:30:00Z")));
    }

    #[test]
    fn next_after_is_strict_and_returns_utc_minute() {
        let schedule = CronSchedule::parse("0 9 * * *").unwrap();
        assert_eq!(
            schedule.next_after(&utc("2026-06-01T09:00:00Z")),
            Some(utc("2026-06-02T09:00:00Z"))
        );
        assert_eq!(
            schedule.next_after(&utc("2026-06-01T09:00:45Z")),
            Some(utc("2026-06-02T09:00:00Z"))
        );
    }

    #[test]
    fn applies_classic_or_semantics_when_both_day_fields_are_restricted() {
        let schedule = CronSchedule::parse("0 0 1 * 1").unwrap();
        assert_eq!(
            schedule.next_after(&utc("2026-06-02T00:00:00Z")),
            Some(utc("2026-06-08T00:00:00Z"))
        );
    }

    #[test]
    fn sunday_zero_and_seven_are_equivalent() {
        let zero = CronSchedule::parse("0 9 * * 0").unwrap();
        let seven = CronSchedule::parse("0 9 * * 7").unwrap();
        let sunday = utc("2026-06-07T09:00:00Z");
        assert!(zero.matches_at(&sunday));
        assert!(seven.matches_at(&sunday));
    }

    #[test]
    fn rejects_wrong_field_count_and_out_of_range_values() {
        assert_eq!(
            CronSchedule::parse("0 0 * * * *"),
            Err(CronError::FieldCount)
        );
        assert!(matches!(
            CronSchedule::parse("60 * * * *"),
            Err(CronError::InvalidField {
                field: "minute",
                ..
            })
        ));
        assert!(CronSchedule::parse("*/0 * * * *").is_err());
        assert!(CronSchedule::parse("5-2 * * * *").is_err());
        assert!(CronSchedule::parse("noon * * * *").is_err());
    }
}
