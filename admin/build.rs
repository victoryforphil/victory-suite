fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("admin-proto/pubsub_admin.proto")?;
    Ok(())
}
