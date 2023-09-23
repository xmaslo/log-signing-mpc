mod create_communication_channel;
pub use create_communication_channel::{Db};

mod rocket_instances;
pub use rocket_instances::{rocket_with_client_auth, rocket_without_client_auth, ServerIdState, SharedDb};

mod pub_endpoints;
pub use pub_endpoints::{rocket_uri_macro_key_gen, rocket_uri_macro_sign, rocket_uri_macro_verify};