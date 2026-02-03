use crate::utils::structs::TodoEventLight;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc, Weekday};
use std::collections::HashMap;
use uuid::Uuid;

//nimmt eingabe vector von todos und erstellt anhand der Rrules einen Vector mit den Wiederholenden Todos, beachtet auch übersprungene und verschobene Todo wiederholungen
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
                    //inesertet das exception event mit parentId und overrides datetime
                    exceptions.insert((parent_id.clone(), date_key.clone()), todo);
                }
            }
        } else {
            // Sonst in Masters
            masters.push(todo);
        }
    }

    // über Master itterieren
    for master in masters {
        //ist es master ohne rrule einfach in Ausgabevektor rein
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
                    rec_instance.id = Uuid::new_v4().to_string(); //Richtige neue id setzt Supabase dann wenn man etwas ändert, hierfür erstmal eine temp generieren lassen für Typesafety oder nochmal referenzierbar sein soll
                    rec_instance.due_datetime = Some(current_date_in_loop_as_str); //wiederholung kriegt aktuelles datum der itteration
                    rec_instance.recurrence_id = Some(Uuid::nil().to_string()); //Null setzen, setzt supabase dann
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

//Helper um nächste Datumsinstanzen nach rrule zu finden

//generiert das nächste Datum einer wiederholenden INstanz
fn calculate_next_date(
    current: DateTime<Utc>,
    rrule: &str,
    start_date_of_rec: DateTime<Utc>,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    match rrule {
        "daily" => Ok(current + Duration::days(1)),
        "weekly" => Ok(current + Duration::weeks(1)),
        "fortnight" => Ok(current + Duration::weeks(2)),
        "weekdays" => {
            //wenn Freitag oder Samstag auf Monatag,
            let next_date = match current.weekday() {
                Weekday::Fri => current + Duration::days(3),
                Weekday::Sat => current + Duration::days(2),
                _ => current + Duration::days(1),
            };
            Ok(next_date)
        }
        "monthly_on_date" => add_months_same_date(current, 1, start_date_of_rec.day()), //Start date mitgeben um Probleme um 31. des Monats zu handeln
        "monthly_on_weekday" => add_month_on_same_weekday(current),
        "annual" => add_months_same_date(current, 12, start_date_of_rec.day()), //Start date mitgeben um Probleme um 31. des Monats zu handeln
        _ => Err(format!("No matching rrule").into()),
    }
}

//für monatlich wiederholende Todos auf den selben Tag
fn add_months_same_date(
    date: DateTime<Utc>,
    months_to_add: u32,
    preferered_day_on_exception: u32,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let year = date.year() + (date.month() as i32 + months_to_add as i32 - 1) / 12;
    let month = (date.month() as i32 + months_to_add as i32 - 1) % 12 + 1;

    //falls wiederholendes todo in nächsten monat fallen würde letzten Tag des monats suchen (bsp. Wdh. Event am 31. März soll dann 30.April nicht 1. Mai) ...
    let day_raw = handle_last_day_of_month(year, month as u32)
        .ok_or_else(|| format!("Date conversion invalid in fn add_months_same_date"))?;

    //...und das kleinere der beiden nehmen
    let day = std::cmp::min(preferered_day_on_exception, day_raw);
    //println!("Fehler bei {} {} {}", year, month, day);
    date.with_day(1) //erst Tag auf eins setzen sonst wirft es manchmal fehler
        .ok_or("Error resetting day to 1")?
        .with_year(year)
        .ok_or("Invalid year in fn add_months_same_date")?
        .with_month(month as u32)
        .ok_or("Invalid month in fn add_months_same_date")?
        .with_day(day)
        .ok_or("Invalid day in fn add_months_same_date")
        .map_err(|e| e.into())
}

//gibt den letzten Tag des Monats aus, bzw. Anzahl an Tage in dem Monat
fn handle_last_day_of_month(year: i32, month: u32) -> Option<u32> {
    if month == 12 {
        // im Dezember -> ersten Tag des neuen Jahres nehmen und davon dann vorgänger
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
            .pred_opt() // Gibt Vorgängerdatum
            .map(|d| d.day())
    } else {
        // alle anderen Monate
        NaiveDate::from_ymd_opt(year, month + 1, 1)? //erster Tag des Nächsten Monats
            .pred_opt() //Davon Vorgängerdatum
            .map(|d| d.day())
    }
}

fn add_month_on_same_weekday(
    date: DateTime<Utc>,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let weekday = date.weekday();
    let day_date = date.day();
    //der wie vielte dieses Wochentages ist es im Monat (3ter Freitag im Monat)
    let nth_weekday_of_month = (day_date - 1) / 7 + 1;

    let (next_year, next_month) = if date.month() == 12 {
        // im Dezember
        (date.year() + 1 /*Jahr+1*/, 1 /*Januar*/)
    } else {
        //sonst nur monat+1
        (date.year(), date.month() + 1)
    };

    let mut date_result = date
        .with_year(next_year)
        .ok_or("Invalid year in fn add_month_on_same_weekday")?
        .with_month(next_month)
        .ok_or("Invalid month in fn add_month_on_same_weekday")?
        .with_day(1)
        .ok_or("Invalid day in fn add_month_on_same_weekday")?;

    //Sucht den ersten Wochentag des Monats, der gleich des Todo DueDate-Tages ist
    while date_result.weekday() != weekday {
        date_result = date_result + Duration::days(1);
    }
    //springt auf n-ten Wochentag des Monats vor
    date_result = date_result + Duration::weeks((nth_weekday_of_month - 1) as i64);

    //Wenn nächster passender Wochentag erst im übernächsten monat eine woche zurück gehen, letzten passenden Wochentag nehmen
    if date_result.month() != next_month {
        date_result = date_result - Duration::weeks(1);
    }

    Ok(date_result) // Gibt das berechnete Datum zurück
}
