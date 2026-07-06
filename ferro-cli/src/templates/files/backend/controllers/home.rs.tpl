use ferro::{Inertia, InertiaProps, Request, Response};

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
    pub message: String,
}

pub async fn index(req: Request) -> Response {
    Inertia::render(
        &req,
        "Home",
        HomeProps {
            title: "Welcome to Ferro!".to_string(),
            message: "Your Inertia + React app is ready.".to_string(),
        },
    )
}
