//! Calcul de calendrier pur pour les flux prévisionnels récurrents.
//! Aucun accès base de données, aucun effet de bord et aucune règle comptable/fiscale.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fréquence déclarée par l'utilisateur sur une prévision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrequenceFlux {
    /// Une seule échéance à la date de départ.
    Ponctuel,
    Mensuel,
    Trimestriel,
    Annuel,
}

impl Default for FrequenceFlux {
    fn default() -> Self {
        Self::Ponctuel
    }
}

impl FrequenceFlux {
    fn intervalle_mois(self) -> Option<u32> {
        match self {
            Self::Ponctuel => None,
            Self::Mensuel => Some(1),
            Self::Trimestriel => Some(3),
            Self::Annuel => Some(12),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ErreurRecurrence {
    #[error("La date de fin ne peut pas être antérieure à la première échéance.")]
    FinAvantDebut,
    #[error("La limite d'occurrences doit être supérieure à zéro.")]
    LimiteNulle,
    #[error("La récurrence dépasse la limite de {limite} occurrences.")]
    TropOccurrences { limite: usize },
    #[error("La date calculée sort de la plage prise en charge.")]
    DateHorsPlage,
}

/// Génère une série d'échéances jusqu'à la borne inclusive la plus proche entre
/// `date_fin` et `horizon`. Les occurrences sont ancrées sur `debut` pour
/// éviter la dérive des échéances en fin de mois.
///
/// Pour une série qui démarre le 31 janvier, les dates obtenues sont par exemple
/// le 28/29 février puis le 31 mars (et non le 28/29 mars).
///
/// `limite` protège l'appelant contre une série gigantesque. La fonction est
/// déterministe : même entrée, mêmes dates, sans dépendance à l'heure système.
pub fn generer_echeances(
    frequence: FrequenceFlux,
    debut: NaiveDate,
    date_fin: Option<NaiveDate>,
    horizon: NaiveDate,
    limite: usize,
) -> Result<Vec<NaiveDate>, ErreurRecurrence> {
    if limite == 0 {
        return Err(ErreurRecurrence::LimiteNulle);
    }
    if date_fin.is_some_and(|fin| fin < debut) {
        return Err(ErreurRecurrence::FinAvantDebut);
    }

    // Une prévision ponctuelle n'est pas bornée par l'horizon de récurrence.
    // Elle doit exister même si sa date est située à plus de 36 mois.
    let Some(intervalle_mois) = frequence.intervalle_mois() else {
        return Ok(vec![debut]);
    };

    let borne = date_fin.map_or(horizon, |fin| fin.min(horizon));
    if borne < debut {
        return Ok(Vec::new());
    }

    let mut dates = Vec::new();
    let mut indice = 0_u32;
    loop {
        let decalage = intervalle_mois
            .checked_mul(indice)
            .ok_or(ErreurRecurrence::DateHorsPlage)?;
        let occurrence = date_ancree(debut, decalage)?;
        if occurrence > borne {
            break;
        }
        if dates.len() >= limite {
            return Err(ErreurRecurrence::TropOccurrences { limite });
        }
        dates.push(occurrence);
        indice = indice
            .checked_add(1)
            .ok_or(ErreurRecurrence::DateHorsPlage)?;
    }
    Ok(dates)
}

/// Calcule chaque occurrence à partir de la date d'origine, jamais à partir de
/// l'occurrence précédente, pour préserver le jour choisi par l'utilisateur.
fn date_ancree(debut: NaiveDate, decalage_mois: u32) -> Result<NaiveDate, ErreurRecurrence> {
    let mois_absolu =
        i64::from(debut.year()) * 12 + i64::from(debut.month0()) + i64::from(decalage_mois);
    let annee =
        i32::try_from(mois_absolu.div_euclid(12)).map_err(|_| ErreurRecurrence::DateHorsPlage)?;
    let mois = u32::try_from(mois_absolu.rem_euclid(12) + 1)
        .map_err(|_| ErreurRecurrence::DateHorsPlage)?;

    let (annee_suivante, mois_suivant) = if mois == 12 {
        (
            annee
                .checked_add(1)
                .ok_or(ErreurRecurrence::DateHorsPlage)?,
            1,
        )
    } else {
        (annee, mois + 1)
    };
    let premier_mois_suivant = NaiveDate::from_ymd_opt(annee_suivante, mois_suivant, 1)
        .ok_or(ErreurRecurrence::DateHorsPlage)?;
    let dernier_jour = (premier_mois_suivant - Duration::days(1)).day();
    NaiveDate::from_ymd_opt(annee, mois, debut.day().min(dernier_jour))
        .ok_or(ErreurRecurrence::DateHorsPlage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn ponctuel_ne_genere_qu_une_echeance() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Ponctuel,
                date("2026-01-31"),
                None,
                date("2028-12-31"),
                1,
            ),
            Ok(vec![date("2026-01-31")])
        );
    }

    #[test]
    fn ponctuel_reste_valide_meme_apres_l_horizon_de_recurrence() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Ponctuel,
                date("2032-01-31"),
                None,
                date("2029-12-31"),
                1,
            ),
            Ok(vec![date("2032-01-31")])
        );
    }

    #[test]
    fn mensuel_fin_de_mois_ne_derive_pas() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-01-31"),
                None,
                date("2026-04-30"),
                10,
            ),
            Ok(vec![
                date("2026-01-31"),
                date("2026-02-28"),
                date("2026-03-31"),
                date("2026-04-30"),
            ])
        );
    }

    #[test]
    fn trimestriel_reste_ancre_sur_le_jour_de_depart() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Trimestriel,
                date("2026-01-31"),
                None,
                date("2026-10-31"),
                4,
            ),
            Ok(vec![
                date("2026-01-31"),
                date("2026-04-30"),
                date("2026-07-31"),
                date("2026-10-31"),
            ])
        );
    }

    #[test]
    fn annuel_gere_les_annees_bissextiles() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Annuel,
                date("2024-02-29"),
                Some(date("2028-02-29")),
                date("2030-12-31"),
                10,
            ),
            Ok(vec![
                date("2024-02-29"),
                date("2025-02-28"),
                date("2026-02-28"),
                date("2027-02-28"),
                date("2028-02-29"),
            ])
        );
    }

    #[test]
    fn date_fin_est_incluse_et_borne_la_serie() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-01-31"),
                Some(date("2026-03-30")),
                date("2026-12-31"),
                10,
            ),
            Ok(vec![date("2026-01-31"), date("2026-02-28")])
        );
    }

    #[test]
    fn horizon_avant_le_debut_ne_genere_aucune_echeance() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-05-01"),
                None,
                date("2026-04-30"),
                10,
            ),
            Ok(Vec::new())
        );
    }

    #[test]
    fn refuse_date_de_fin_invalide_limite_nulle_et_trop_d_occurrences() {
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-05-01"),
                Some(date("2026-04-30")),
                date("2026-12-31"),
                10,
            ),
            Err(ErreurRecurrence::FinAvantDebut)
        );
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-01-01"),
                None,
                date("2026-12-31"),
                0,
            ),
            Err(ErreurRecurrence::LimiteNulle)
        );
        assert_eq!(
            generer_echeances(
                FrequenceFlux::Mensuel,
                date("2026-01-01"),
                None,
                date("2026-12-31"),
                3,
            ),
            Err(ErreurRecurrence::TropOccurrences { limite: 3 })
        );
    }
}
