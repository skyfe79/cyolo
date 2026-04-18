//! Tests that exercise the `profile` dispatcher (clap routing) itself,
//! not any single subcommand's internals. One concept per file.

mod test_dispatch_no_args_shows_help;
mod test_dispatch_unknown_subcommand_returns_error;
mod test_dispatch_routes_to_default;
mod test_dispatch_routes_to_init_with_unknown_name;
mod test_dispatch_routes_to_sync_mcp_with_unknown_name;
mod test_dispatch_rejects_extra_args_on_current;
