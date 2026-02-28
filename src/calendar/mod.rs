//! Functions for handling Calendar Events
//! 
//! Calendar events are built in a loos simillarity to RFC 5545.
//! An event can either be a single event, recurrent or an exception to a recurrent event. No event can be an recurrence exception and recurrent itself.
//! Every change (creation, editing or deletion) of an event is done by changing the entry in the supabase database, syncing it to the local database which is then displayed.

pub mod backend;
pub mod frontend;