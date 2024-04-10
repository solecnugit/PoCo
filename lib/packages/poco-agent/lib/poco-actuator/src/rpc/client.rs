use tokio::sync::mpsc;

use crate::{config::RpcTaskConfig, TaskActuator};
use tonic::Request;
use transcode::DispatchVoDRequest;
use poco_types::types::task::{OnChainTaskConfig, TaskInputSource};

use transcode::transcoder_client::TranscoderClient;
// use hello_world::HelloRequest;

pub mod transcode {
    tonic::include_proto!("transcode");
}

pub async fn send_rpc_request(task: RpcTaskConfig, tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {

    // todo!();

    let channel = tonic::transport::Channel::from_static("http://192.168.122.45:50051")
        .connect()
        .await?;

    let mut client = TranscoderClient::new(channel);

    let ipfsurl = match task.input {
        TaskInputSource::Ipfs { hash} => hash,
        TaskInputSource::Link { url } => panic!("Not implemented yet"),
    };

    let vcodec = match task.config["target"]["video"]["codec"].as_str() {
        Some("H.265") => 1,
        Some("H.264") => 0,
        _ => panic!("Not implemented yet"),
    };

    // if 

    let request = DispatchVoDRequest {
        taskid: task.taskid.to_string(),
        originurl: ipfsurl,
        outputcodec: vcodec,
        uniqueid: "".to_string()
    };  

    let mut response_stream = client.dispatch_vo_d_task(Request::new(request)).await?.into_inner();

    // Iterate over the stream of responses.
    while let Some(response) = response_stream.message().await? {
        // Handle each response.
        println!("Response: {:?}", response);

        tx.send(format!("Received a response: {:?}", response)).await?;
    }

    Ok(())

    }

    // let request = tonic::Request::new(DispatchVoDRequest {
    //     source: "http://

    // let request = tonic::Request::new(HelloRequest {
    //     name: "Tonic".into(),
    // });

    // let response = client.say_hello(request).await?;

    // println!("RESPONSE={:?}", response);

    // Ok(())
// }