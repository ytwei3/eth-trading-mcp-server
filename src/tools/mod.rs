pub mod get_balance;
pub mod get_token_price;
pub mod swap_tokens;

use crate::types::Tool;

pub fn get_all_tools() -> Vec<Tool> {
    vec![
        get_balance::get_tool_definition(),
        get_token_price::get_tool_definition(),
        swap_tokens::get_tool_definition(),
    ]
}
