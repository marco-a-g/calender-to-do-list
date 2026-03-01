//! Function for expanding recurrent events for display in the frontend.

use crate::utils::{date_handling::calculate_next_date, structs::CalendarEventLight};
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Expands the recurrent events to all single instances within the year to be displayed.
///
/// Creates all missing Instances for the recurrent events also handling overriding exceptions.
/// Instances are created with new Ids for easier handling in the frontend. These must not be used for manipulating the events.
///
/// ## Arguments
/// - `events`- a vector of `CalendarEventLight` as taken out of the local database.
/// - `year`- The year for which the recurrent events are expanded. If no year is provided, the current year is used.
///
pub fn expand_recurring_events(
    events: Vec<CalendarEventLight>,
    year: Option<chrono::DateTime<Utc>>, // defaults to now
) -> Result<Vec<CalendarEventLight>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut masters: Vec<CalendarEventLight> = Vec::new();
    // Set aus Key, value pairs mit RecId und overridesDatetime, und event selbst soll performanter sein, laut LLM
    let mut exceptions: HashMap<(String, String), CalendarEventLight> = HashMap::new();

    //Betrachtungszeitpunkt setzen:
    let mut now = chrono::Utc::now();
    if let Some(date) = year {
        now = date
    }

    //Trennung in Master und Exception
    for event in events {
        // hat es rec id -> ist exception
        let is_exception = match &event.recurrence_id {
            Some(_id) => true,
            None => false,
        };
        if is_exception {
            // Es ist eine Exception, dann mit master-id und überschreibendem Datum speichern in Hashmap
            if let Some(ref parent_id) = event.recurrence_id {
                if let Some(ref date_key) = event.overrides_datetime {
                    let normalized_date_key = DateTime::parse_from_rfc3339(date_key)
                        .map(|d| d.with_timezone(&Utc).to_rfc3339())
                        .unwrap_or_else(|_| date_key.clone());
                    //inesertet das exception event mit parentId und overrides datetime
                    exceptions.insert((parent_id.clone(), normalized_date_key), event);
                }
            }
        } else {
            // Sonst in Masters
            masters.push(event);
        }
    }

    for master in masters {
        //ist es master ohne rrule (:= nicht wiederholendes event) einfach in Ausgabevektor rein
        if master.rrule.is_none() {
            result.push(master);
            continue;
        }

        //rrule, from_dt aus master extrahieren für weiteren Bearbeitung
        let rrule = master.rrule.clone().ok_or("RRule missing")?;
        let from_dt = master.from_date_time.clone();

        //to_date_time parsen
        let to_dt = master.to_date_time.clone().and_then(|d| {
            DateTime::parse_from_rfc3339(&d)
                .map(|t| t.with_timezone(&Utc))
                .ok()
        });

        //String von from_dt in Datetime parsen
        let start_date_of_recurrance =
            DateTime::parse_from_rfc3339(&from_dt).map(|d| d.with_timezone(&Utc))?;

        //Dauer des einzelevents berechnen
        let duration = to_dt.map(|t| t - start_date_of_recurrance);

        // Until date in DateTime parsen, wenn vorhanden, sonst None setzen
        let end_date_of_recurrance = match master.recurrence_until.clone() {
            Some(master_rec) => Some(
                DateTime::parse_from_rfc3339(&master_rec)
                    .map(|date| date.with_timezone(&Utc))
                    .map(|date_utc: DateTime<Utc>| {
                        date_utc
                            .with_hour(23)
                            .unwrap_or(date_utc)
                            .with_minute(59)
                            .unwrap_or(date_utc)
                            .with_second(59)
                            .unwrap_or(date_utc)
                    })?,
            ),
            None => None, //eigentlich nur Fallback, sollte immer eins existieren
        };

        //Start zum itterieren setzen
        let mut current_date = start_date_of_recurrance;
        let mut current_end = to_dt;
        let mut first_iter = true;

        //Über die möglichen Wiederholungen der nächsten 2 Jahre itterieren
        loop {
            //ist end datum erreicht => aufhören
            if let Some(rec_end) = end_date_of_recurrance {
                if current_date > rec_end {
                    break;
                }
            }
            //sind Wiederholungsinstanzen bis 2 Jahre in die Zukunft erreicht? => aufhören
            if current_date > (now.checked_add_months(chrono::Months::new(24)).unwrap()) {
                break;
            }

            //aktuelle Daten des zu itterierenden Intervalls zwischenspeichern
            let current_date_in_loop_as_str = current_date.to_rfc3339();
            let current_end_in_loop_as_str = current_end.map(|e| e.to_rfc3339());
            //Key für K-V-Pair für schnelleres suchen in exceptions, aus master-id und master-datum
            let lookup_key = (master.id.clone(), current_date_in_loop_as_str.clone());

            //Gibt es für dieses Event eine exception in K-V-Store => exception behandeln
            if let Some(exception) = exceptions.get(&lookup_key) {
                // ist geskipped => keine Fake instzanz von dem event pushen
                //ist es nicht geskipped => pushen in result
                if !exception.skipped {
                    result.push(exception.clone());
                }
                first_iter = false;
            } else {
                //Keine exception => Instanz erzeugen
                //erster Durchlauf => Master ist die Instanz
                if first_iter {
                    // nichts ändern einfach in result rein
                    let master_instance = master.clone();
                    result.push(master_instance);
                    first_iter = false;
                } else {
                    // hier sind Wiederholungen
                    let mut rec_instance = master.clone();
                    rec_instance.id = Uuid::new_v4().to_string(); //Richtige neue id setzt Supabase dann wenn man etwas ändert, hierfür erstmal eine temp generieren lassen für Typesafety oder nochmal referenzierbar sein soll
                    rec_instance.from_date_time = current_date_in_loop_as_str; //wiederholung kriegt aktuelles datum der itteration
                    rec_instance.to_date_time = current_end_in_loop_as_str;
                    rec_instance.recurrence_id = Some(master.id.clone());
                    rec_instance.rrule = None; //Recurrance instanzen haben selber keine rrule
                    rec_instance.recurrence_until = None; //recurrance instanzen haben selber kein until datum
                    result.push(rec_instance);
                }
            }
            //aktuell betrachtete Daten weiterstellen je nach rrule
            current_date = calculate_next_date(current_date, &rrule, start_date_of_recurrance)?;
            current_end =
                current_end.and_then(|ce| calculate_next_date(ce, &rrule, to_dt.unwrap()).ok());
        }
    }
    //Ergebnisvektor aus Mastern und "Fake"-events ausgeben
    Ok(result)
}
