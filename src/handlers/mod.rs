pub mod auth;
pub mod containers;
pub mod deploy;
pub mod health;
pub mod images;
pub mod projects;
pub mod system;

pub use auth::auth_routes;
pub use containers::container_routes;
pub use deploy::{deploy_routes, streaming_routes};
pub use health::health_routes;
pub use images::image_routes;
pub use projects::project_routes;
pub use system::system_routes;
