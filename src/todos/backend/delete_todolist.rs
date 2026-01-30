use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};
use dioxus::prelude::*;

//#[server]
pub async fn delete_todo_list(list: TodoListLight) -> Result<(), ServerFnError> {
    //Hier Insert zu Remote-DB
    println!(
        "Delete Todo-List Server Funktion wurde aufgerufen mit: {:?}",
        list
    );
    //damit Server function akzeptiert wird-----
    let x = 1;
    match x {
        1 => Ok(()),
        _ => Err(ServerFnError::new("")),
    }
    //-------------------------------------------
}
