//! Very simple echoing contract which just returns incoming string if any,
//! but performing sub call of given message to test response.
//!
//! Additionally, it bypasses all events and attributes send to it.

use crate::{Contract, ContractWrapper};
use cosmwasm_std::{
    to_json_binary, Attribute, Binary, CustomMsg, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    Reply, Response, StdError, SubMsg, SubMsgResponse, SubMsgResult,
};
use cw_utils::{parse_execute_response_data, parse_instantiate_response_data};
use derivative::Derivative;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Choosing a reply id less than ECHO_EXECUTE_BASE_ID indicates an Instantiate message reply by convention.
// An Execute message reply otherwise.
pub const EXECUTE_REPLY_BASE_ID: u64 = i64::MAX as u64;

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default(bound = "", new = "true"))]
pub struct Message<ExecC>
where
    ExecC: Debug + PartialEq + Clone + JsonSchema + 'static,
{
    pub data: Option<String>,
    pub sub_msg: Vec<SubMsg<ExecC>>,
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
}

// This can take some data... but happy to accept {}
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default(bound = "", new = "true"))]
pub struct InitMessage<ExecC>
where
    ExecC: Debug + PartialEq + Clone + JsonSchema + 'static,
{
    pub data: Option<String>,
    pub sub_msg: Option<Vec<SubMsg<ExecC>>>,
}

#[allow(clippy::unnecessary_wraps)]
fn instantiate<ExecC>(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMessage<ExecC>,
) -> Result<Response<ExecC>, StdError>
where
    ExecC: Debug + PartialEq + Clone + JsonSchema + 'static,
{
    let mut res = Response::new();
    if let Some(data) = msg.data {
        res = res.set_data(data.into_bytes());
    }
    if let Some(msgs) = msg.sub_msg {
        res = res.add_submessages(msgs);
    }
    Ok(res)
}

#[allow(clippy::unnecessary_wraps)]
fn execute<ExecC>(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: Message<ExecC>,
) -> Result<Response<ExecC>, StdError>
where
    ExecC: Debug + PartialEq + Clone + JsonSchema + 'static,
{
    let mut resp = Response::new();

    if let Some(data) = msg.data {
        resp = resp.set_data(data.into_bytes());
    }

    Ok(resp
        .add_submessages(msg.sub_msg)
        .add_attributes(msg.attributes)
        .add_events(msg.events))
}

fn query(_deps: Deps, _env: Env, msg: Empty) -> Result<Binary, StdError> {
    to_json_binary(&msg)
}

#[allow(clippy::unnecessary_wraps)]
fn reply<ExecC>(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response<ExecC>, StdError>
where
    ExecC: Debug + PartialEq + Clone + JsonSchema + 'static,
{
    let res = Response::new();
    if let Reply {
        id,
        result: SubMsgResult::Ok(SubMsgResponse {
            data: Some(data), ..
        }),
    } = msg
    {
        // We parse out the WasmMsg::Execute wrapper...
        // TODO: Handle all of Execute, Instantiate, and BankMsg replies differently.
        let parsed_data = if id < EXECUTE_REPLY_BASE_ID {
            parse_instantiate_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(e.to_string()))?
                .data
        } else {
            parse_execute_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(e.to_string()))?
                .data
        };

        if let Some(data) = parsed_data {
            Ok(res.set_data(data))
        } else {
            Ok(res)
        }
    } else {
        Ok(res)
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute::<Empty>, instantiate::<Empty>, query)
        .with_reply(reply::<Empty>);
    Box::new(contract)
}

pub fn custom_contract<C>() -> Box<dyn Contract<C>>
where
    C: CustomMsg + DeserializeOwned + 'static,
{
    let contract =
        ContractWrapper::new(execute::<C>, instantiate::<C>, query).with_reply(reply::<C>);
    Box::new(contract)
}
