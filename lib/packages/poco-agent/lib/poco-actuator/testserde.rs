use poco_actuator::config::{RawTaskConfigFile, RawTaskInputSource};
use poco_actuator::TaskConfigFactory;
use poco_actuator::media::MediaTranscodingActuator;
use borsh::de::BorshDeserialize;
fn main(){
    let vec_u8: Vec<u8> = vec![1, 2, 3, 4, 5];
    let vec_value: Vec<serde_json::Value> = vec_u8.into_iter().map(|x| serde_json::Value::Number(x.into())).collect();
    let value = serde_json::Value::Array(vec_value);
    println!("{}", value);


    let bytes: Vec<u8> = vec![5, 0,  0,  0, 72, 46, 50, 54, 52,
    3, 0,  0,  0, 65, 65, 67,  5,  0,
    0, 0, 72, 46, 50, 54, 53,  3,  0,
    0, 0, 65, 65, 67]; // 这里是你的字节向量
    let config: <MediaTranscodingActuator as TaskConfigFactory>::Config = <MediaTranscodingActuator as TaskConfigFactory>::Config::deserialize(&mut &*bytes).unwrap();
    println!("{:?}", config);
    // let config: RawTaskConfigFile = serde_json::from_slice(&bytes).unwrap();
}