use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Todo {
    pub title: String,
    pub is_done: bool,
    pub created_block: u64,
    pub updated_block: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TodoList {
    pub list: Vec<Todo>,
}

pub const TODOS: Item<TodoList> = Item::new("todolist");
