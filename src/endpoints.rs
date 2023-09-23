mod create_communication_channel;
pub use create_communication_channel::{Db};

mod rocket_instances;
pub use rocket_instances::{rocket_with_client_auth, rocket_without_client_auth, ServerIdState, SharedDb};