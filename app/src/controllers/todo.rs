use ferro::{handler, json_response, App, Response, ResponseExt};

use crate::actions::todo_action::{CreateRandomTodoAction, ListTodosAction};

#[handler]
pub async fn create_random() -> Response {
    let action = App::resolve::<CreateRandomTodoAction>()?;

    match action.execute().await {
        Ok(todo) => json_response!({
            "success": true,
            "todo": todo
        })
        .status(201),
        Err(e) => json_response!({
            "success": false,
            "error": e.to_string()
        })
        .status(500),
    }
}

#[handler]
pub async fn list() -> Response {
    let action = App::resolve::<ListTodosAction>()?;

    match action.execute().await {
        Ok(todos) => json_response!({
            "success": true,
            "todos": todos
        }),
        Err(e) => json_response!({
            "success": false,
            "error": e.to_string()
        })
        .status(500),
    }
}
