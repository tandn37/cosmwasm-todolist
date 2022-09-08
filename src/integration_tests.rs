#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {  };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }
    mod count {
        use super::*;
        use crate::{msg::{ExecuteMsg, QueryMsg}, state::Todo};

        fn get_list(app: &App, cw_template_contract: &CwTemplateContract) -> Vec<Todo> {
            let list_msg = QueryMsg::List {  };
            let todos: Vec<Todo> = app
                .wrap()
                .query_wasm_smart(cw_template_contract.addr(), &list_msg)
                .unwrap();
            return todos;
        }
        #[test]
        fn all() {
            let (mut app, cw_template_contract) = proper_instantiate();
            let msg = ExecuteMsg::Add { title: "test".to_string() };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let _res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
            
            let todos = get_list(&app, &cw_template_contract);
            assert_eq!("test", todos[0].title);
            assert_eq!(false, todos[0].is_done);

            let msg = ExecuteMsg::Update { id: 1 };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
            let todos = get_list(&app, &cw_template_contract);
            assert_eq!(true, todos[0].is_done);

            let msg = ExecuteMsg::Remove { id: 1 };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
            let todos = get_list(&app, &cw_template_contract);
            assert_eq!(Vec::<Todo>::new(), todos);
        }
    }
}
