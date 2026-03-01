use crate::utils::date_handling::calculate_next_date;
use crate::utils::structs::TodoEventLight;
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;

//nimmt eingabe vector von todos und erstellt anhand der Rrules einen Vector mit den Wiederholenden Todos, beachtet auch übersprungene und verschobene Todo wiederholungen
/// Expands a list of to-dos by generating individual instances for recurring tasks.
///
/// Separates master recurring events from their exceptions. Then iterates over the master events and generates "fake" instances for future occurrences based on their recurrence rule.
/// Expansion currently capped at 2 years (730 days).
///
/// Exceptions are mapped back to their respective dates, overriding or "hiding" the generated virtual instances.
///
/// ## Arguments
///
/// * `todos` - Vector of `TodoEventLight`.
///
/// ## Errors
///
/// Returns a boxed dynamic error if datetime parsing fails or the recurrence rule calculation (`calculate_next_date`) encounters an invalid pattern.
pub fn expand_recurring_todos(
    todos: Vec<TodoEventLight>,
) -> Result<Vec<TodoEventLight>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut masters: Vec<TodoEventLight> = Vec::new();
    // Set aus Key, value pairs mit RecId und overridesDatetime, und todo selbst soll performanter sein, laut LLM
    let mut exceptions: HashMap<(String, String), TodoEventLight> = HashMap::new();

    //Trennung in Master und Exception
    for todo in todos {
        // hat es rec id -> ist exception
        let is_exception = match &todo.recurrence_id {
            Some(_id) => true,
            None => false,
        };
        if is_exception {
            // Es ist eine Exception, dann mit master-id und überschreibendem DAtum speichern in Hashmap
            if let Some(ref parent_id) = todo.recurrence_id {
                if let Some(ref date_key) = todo.overrides_datetime {
                    //Github PR Review Fix, für evtl. Vergleichsprobleme bei unterschiedlichen Datumsformaten, umparsen damit Vergleich klappt
                    let normalized_date_key = DateTime::parse_from_rfc3339(date_key)
                        .map(|d| d.with_timezone(&Utc).to_rfc3339())
                        .unwrap_or_else(|_| date_key.clone());
                    //inesertet das exception event mit parentId und overrides datetime
                    exceptions.insert((parent_id.clone(), normalized_date_key), todo);
                }
            }
        } else {
            // Sonst in Masters
            masters.push(todo);
        }
    }

    // über Master itterieren
    for master in masters {
        //ist es master ohne rrule (:= nicht wiederholendes ToDo) einfach in Ausgabevektor rein
        if master.rrule.is_none() {
            result.push(master);
            continue;
        }

        //rrule aus master extrahieren für weiteren Bearbeitung
        let rrule = master.rrule.clone().ok_or("RRule missing")?; //sollte eig nur mit clone passen

        //start der Recurrance als String aus master holen, wenn vorhanden, sonst in result pushen direkt
        let start_date_od_reccurance_str = match master.due_datetime.clone() {
            Some(d) => d,
            None => {
                result.push(master);
                continue;
            }
        };

        //String von start of rec in Datetime parsen
        let start_date_of_recurrance = DateTime::parse_from_rfc3339(&start_date_od_reccurance_str)
            .map(|d| d.with_timezone(&Utc))?;

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

        //maximal recurrance vorerst mal auf 2 Jahre beschränken
        let max_instances = 730;
        //Zählervar für Annäherung an max_instances
        let mut count = 0;
        //Start zum itterieren setzen
        let mut current_date = start_date_of_recurrance;

        //Über die möglichen Wiederholungen der nächsten 2 Jahre itterieren
        loop {
            //ist end datum erreicht => aufhören
            if let Some(rec_end) = end_date_of_recurrance {
                if current_date > rec_end {
                    break;
                }
            }
            //sind 2 Jahre Wiederholungsinstanzen erreicht? => aufhören
            if count >= max_instances {
                break;
            }

            //aktuelles Datum des zu itterierenden Intervalls zwischenspiechern
            let current_date_in_loop_as_str = current_date.to_rfc3339();
            //Key für K-V-Pair für schnelleres suchen in exceptions, aus master-id und master-datum
            let lookup_key = (master.id.clone(), current_date_in_loop_as_str.clone());

            //Gibt es für dieses Event eine exception in K-V-Store => exception behandeln
            if let Some(exception) = exceptions.get(&lookup_key) {
                // ist geskipped => keine Fake instzanz von dem Todo pushen
                //ist es nicht geskipped => pushen in result
                if !exception.skipped {
                    result.push(exception.clone());
                }
            } else {
                //"Nicht-exceptions" behandeln
                //hier ist master
                if count == 0 {
                    // nichts ändern einfach in result rein
                    let master_instance = master.clone();
                    result.push(master_instance);
                } else {
                    // hier sind Wiederholungen
                    let mut rec_instance = master.clone();
                    rec_instance.id = master.id.clone(); //Richtige neue id setzt Supabase dann wenn man etwas ändert
                    rec_instance.due_datetime = Some(current_date_in_loop_as_str); //wiederholung kriegt aktuelles datum der itteration
                    rec_instance.recurrence_id = Some(master.id.clone());
                    rec_instance.rrule = None; //Recurrance instanzen haben selber keine rrule
                    rec_instance.recurrence_until = None; //recurrance instanzen haben selber kein until datum
                    result.push(rec_instance);
                }
            }
            //aktuell betrachtetes datum weiterstellen je nach rrule
            current_date = calculate_next_date(current_date, &rrule, start_date_of_recurrance)?;
            //Zähler für Prüfung auf max_instanzes erhöhen
            count += 1;
        }
    }
    //Ergebnisvektor aus Mastern und "Fake"-Todos ausgeben
    Ok(result)
}
