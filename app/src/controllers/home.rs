use ferro::{handler, App, Inertia, InertiaProps, Request, Response};

use crate::actions::example_action::ExampleAction;

#[derive(InertiaProps)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(InertiaProps)]
pub struct Stats {
    pub visits: u32,
    pub likes: u32,
}

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
    pub message: String,
    pub user: User,
    pub stats: Stats,
}

#[handler]
pub async fn index(req: Request) -> Response {
    // Get the action from the service container using resolve()
    // This returns a proper error response if not registered
    let action = App::resolve::<ExampleAction>()?;
    let message = action.execute();

    // Use the new async-safe Inertia::render API
    Inertia::render(
        &req,
        "Home",
        HomeProps {
            title: "Welcome to Ferro!".to_string(),
            message,
            user: User {
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            },
            stats: Stats {
                visits: 1234,
                likes: 567,
            },
        },
    )
}
