use warp::{hyper::StatusCode, reply::with_status, Filter};

use snarker::{
    p2p::{
        connection::incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
        webrtc, PeerId,
    },
    rpc::RpcRequest,
};

use super::rpc::RpcP2pConnectionIncomingResponse;

pub async fn run(port: u16, rpc_sender: super::RpcSender) {
    let signaling = warp::path!("mina" / "webrtc" / "signal")
        .and(warp::post())
        .and(warp::filters::body::json())
        .then(move |offer: webrtc::Offer| {
            let rpc_sender = rpc_sender.clone();
            async move {
                let mut rx = rpc_sender
                    .multishot_request(
                        2,
                        RpcRequest::P2pConnectionIncoming(P2pConnectionIncomingInitOpts {
                            peer_id: PeerId::from_public_key(offer.identity_pub_key.clone()),
                            signaling: IncomingSignalingMethod::Http,
                            offer,
                        }),
                    )
                    .await;

                match rx.recv().await {
                    Some(RpcP2pConnectionIncomingResponse::Answer(answer)) => match answer {
                        Ok(v) => {
                            let resp = serde_json::to_string(&v).unwrap();
                            with_status(resp, StatusCode::from_u16(200).unwrap())
                        }
                        Err(err) => with_status(err, StatusCode::from_u16(400).unwrap()),
                    },
                    _ => with_status("rejected".to_owned(), StatusCode::from_u16(400).unwrap()),
                }
            }
        });
    let routes = signaling;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
