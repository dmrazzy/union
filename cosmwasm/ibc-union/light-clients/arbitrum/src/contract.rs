use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use ibc_union_light_client::{
    msg::{InstantiateMsg, QueryMsg},
    IbcClientError,
};

use crate::client::ArbitrumLightClient;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, IbcClientError<ArbitrumLightClient>> {
    ibc_union_light_client::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<ArbitrumLightClient>(deps, env, msg).map_err(Into::into)
}