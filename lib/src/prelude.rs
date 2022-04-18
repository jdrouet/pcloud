pub trait Command {
    type Output;
    type Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}
