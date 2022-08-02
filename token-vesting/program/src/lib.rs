use bonfida_utils::declare_id_with_central_state;

#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub(crate) mod processor;

declare_id_with_central_state!("4eG2WCq8LiamUW5nzhRhyJUS24UEnM8pDezowJmMC6wM"); //TODO
