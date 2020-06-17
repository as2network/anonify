use std::{sync::Arc, env};
// use std::{sync::Arc, thread, time};
use failure::Error;
// use log::debug;
// use anonify_host::dispatcher::get_state;
use anonify_bc_connector::{
    // EventDB,
    BlockNumDB,
    traits::*,
    // eth::*,
};
use anonify_runtime::Text;
use anonify_host::Dispatcher;
use dx_app::send_invoice;
use actix_web::{
    web,
    HttpResponse,
};
// use anyhow::anyhow;
use sgx_types::sgx_enclave_id_t;

#[derive(Debug)]
pub struct Server<D: Deployer, S: Sender, W: Watcher<WatcherDB=DB>, DB: BlockNumDB> {
    pub eid: sgx_enclave_id_t,
    pub eth_url: String,
    pub abi_path: String,
    pub dispatcher: Dispatcher<D, S, W, DB>,
}

impl<D, S, W, DB> Server<D, S, W, DB>
    where
        D: Deployer,
        S: Sender,
        W: Watcher<WatcherDB=DB>,
        DB: BlockNumDB,
{
    pub fn new(eid: sgx_enclave_id_t) -> Self {
        let eth_url = env::var("ETH_URL").expect("ETH_URL is not set.");
        let abi_path = env::var("ANONYMOUS_ASSET_ABI_PATH").expect("ANONYMOUS_ASSET_ABI_PATH is not set.");
        let event_db = Arc::new(DB::new());
        let dispatcher = Dispatcher::<D, S, W, DB>::new(eid, &eth_url, event_db).unwrap();

        Server {
            eid,
            eth_url,
            abi_path,
            dispatcher,
        }
    }
}

const DEFAULT_SEND_GAS: u64 = 3_000_000;

pub fn handle_send_invoice<D, S, W, DB>(
    server: web::Data<Arc<Server<D, S, W, DB>>>,
    req: web::Json<dx_api::send_invoice::post::Request>,
) -> Result<HttpResponse, Error>
    where
        D: Deployer,
        S: Sender,
        W: Watcher<WatcherDB=DB>,
        DB: BlockNumDB,
{
    let access_right = req.into_access_right()?;
    let signer = server.dispatcher.get_account(0)?;
    let recipient = req.recipient;
    let body = Text::new(req.body.clone().into());
    let body = Text::from(body);

    let send_invoice_state = send_invoice{ recipient, body };

    let receipt = server.dispatcher.send_instruction(
        access_right,
        send_invoice_state,
        req.state_id,
        "send_invoice",
        signer,
        DEFAULT_SEND_GAS,
        &req.contract_addr,
        &server.abi_path,
    )?;

    Ok(HttpResponse::Ok().json(dx_api::send_invoice::post::Response(receipt)))
}