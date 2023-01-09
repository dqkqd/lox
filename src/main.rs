fn main() -> Result<(), Box<dyn std::error::Error>> {
    lox::exec()?;
    Ok(())
}
