// Commands Module: CQRS write side
// Commands represent intentions to change the state
// They're processed by command handlers which emit events

pub mod register_user_command;
pub mod rename_user_command;
pub mod command_handler;

pub use register_user_command::RegisterUserCommand;
pub use rename_user_command::RenameUserCommand;
pub use command_handler::UserCommandHandler;
