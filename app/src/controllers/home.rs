use ferro::{handler, Inertia, InertiaProps, Request, Response};

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
    Inertia::render(
        &req,
        "Home",
        HomeProps {
            title: "Welcome to Ferro!".to_string(),
            message: "Hello from Ferro!".to_string(),
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
