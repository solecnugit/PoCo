fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/rpc/proto/transcoding.proto")?;
    Ok(())
}