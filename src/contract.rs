#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Todo, TODOS, TodoList};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:todolist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_NUMBER_OF_ITEMS: u32 = 1000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TODOS.save(deps.storage, &TodoList {
        list: Vec::new()
    })?;
    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Add { title } => handle_add(deps, env, title),
        ExecuteMsg::Update { id } => handle_update(deps, env, id),
        ExecuteMsg::Remove { id } => handle_remove(deps, id),
    }
}

pub fn handle_add(deps: DepsMut, env: Env, title: String) -> Result<Response, ContractError> {
    if title.is_empty() {
        return Err(ContractError::InvalidParams { val: "Empty content".to_string() });
    }
    
    TODOS.update(deps.storage, |mut state| -> Result<_, ContractError>{
        if state.list.len() > MAX_NUMBER_OF_ITEMS as usize {
            return Err(ContractError::CustomError { val: "List is full".to_string() });
        }
        let todo = Todo {
            title,
            is_done: false,
            created_block: env.block.height,
            updated_block: None,
        };
        state.list.push(todo);
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "add"))
}

pub fn handle_update(deps: DepsMut, env: Env, id: u32) -> Result<Response, ContractError> {
    TODOS.update(deps.storage, |mut state| -> Result<_, ContractError>{
        if id < 1 {
            return Err(ContractError::InvalidParams { val: "invalid id".to_string() });
        }
        let todo_id = (id - 1) as usize;
        let todo = state.list.get(todo_id);
        if todo.is_none() {
            return Err(ContractError::NotFound {});
        }
        let mut new_todo = todo.unwrap().to_owned();
        new_todo.is_done = !new_todo.is_done;
        new_todo.updated_block = Some(env.block.height);
        state.list[todo_id] = new_todo;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "update"))
}

pub fn handle_remove(deps: DepsMut, id: u32) -> Result<Response, ContractError> {
    TODOS.update(deps.storage, |mut state| -> Result<_, ContractError>{
        if id < 1 {
            return Err(ContractError::InvalidParams { val: "invalid id".to_string() });
        }
        let todo_id = (id - 1) as usize;
        let todo = state.list.get(todo_id);
        if todo.is_none() {
            return Err(ContractError::NotFound {  });
        }
        state.list.remove(todo_id);
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "remove"))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::List {} => to_binary(&get_todo_list(deps)?),
    }
}

pub fn get_todo_list(deps: Deps) -> StdResult<Vec<Todo>> {
    let todos = TODOS.load(deps.storage)?;
    Ok(todos.list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let res = instantiate(deps.as_mut(), mock_env(), mock_info("test", &[]), InstantiateMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!(Vec::<Todo>::new(), res);
    }
    #[test]
    fn add() {
        let mut deps = mock_dependencies();
        let _res = instantiate(deps.as_mut(), mock_env(), mock_info("test", &[]), InstantiateMsg {}).unwrap();
        let _res = get_todo_list(deps.as_ref()).unwrap();

        // can not add empty title
        let err = handle_add(deps.as_mut(), mock_env(), "".to_string()).unwrap_err();
        match err {
            ContractError::InvalidParams { val } => {
                assert_eq!("Empty content", val);
            }
            e => panic!("unexpected error: {}", e)
        }

        let _res = handle_add(deps.as_mut(), mock_env(), "test".to_string());
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!("test", res[0].title);
        assert_eq!(false, res[0].is_done);
        assert_eq!(12345, res[0].created_block);
        assert_eq!(None, res[0].updated_block);
    }

    #[test]
    fn update() {
        let mut deps = mock_dependencies();
        let _res = instantiate(deps.as_mut(), mock_env(), mock_info("test", &[]), InstantiateMsg {}).unwrap();
        let _res = handle_add(deps.as_mut(), mock_env(), "test".to_string());
        
        let err = handle_update(deps.as_mut(), mock_env(), 0).unwrap_err();
        match err {
            ContractError::InvalidParams { val } => {
                assert_eq!(val, "invalid id");
            },
            e => panic!("unexpected error: {}", e)
        }

        let err = handle_update(deps.as_mut(), mock_env(), 2).unwrap_err();
        match err {
            ContractError::NotFound {} => {},
            e => panic!("unexpected error: {}", e)
        }
        handle_update(deps.as_mut(), mock_env(), 1).unwrap();
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!("test", res[0].title);
        assert_eq!(true, res[0].is_done);
        assert_eq!(12345, res[0].created_block);
        assert_eq!(Some(12345), res[0].updated_block);
    }

    #[test]
    fn remove() {
        let mut deps = mock_dependencies();
        let _res = instantiate(deps.as_mut(), mock_env(), mock_info("test", &[]), InstantiateMsg {}).unwrap();
        let _res = handle_add(deps.as_mut(), mock_env(), "test".to_string());
        
        let err = handle_remove(deps.as_mut(), 0).unwrap_err();
        match err {
            ContractError::InvalidParams { val } => {
                assert_eq!(val, "invalid id");
            },
            e => panic!("unexpected error: {}", e)
        }

        let err = handle_remove(deps.as_mut(), 2).unwrap_err();
        match err {
            ContractError::NotFound {} => {},
            e => panic!("unexpected error: {}", e)
        }
        handle_remove(deps.as_mut(), 1).unwrap();
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!(Vec::<Todo>::new(), res);
    }
    #[test]
    fn multiple_add() {
        let mut deps = mock_dependencies();
        let _res = instantiate(deps.as_mut(), mock_env(), mock_info("test", &[]), InstantiateMsg {}).unwrap();
        let _res = get_todo_list(deps.as_ref()).unwrap();

        let _res = handle_add(deps.as_mut(), mock_env(), "test".to_string());
        let _res = handle_add(deps.as_mut(), mock_env(), "test2".to_string());

        handle_update(deps.as_mut(), mock_env(), 2).unwrap();
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!(2, res.len());
        assert_eq!(true, res[1].is_done);

        let _res = handle_add(deps.as_mut(), mock_env(), "test3".to_string());
        handle_remove(deps.as_mut(), 2).unwrap();
        let res = get_todo_list(deps.as_ref()).unwrap();
        assert_eq!(2, res.len());
        assert_eq!("test", res[0].title);
        assert_eq!("test3", res[1].title);
    }
}
