#[cfg(feature = "ultimate")]
// pub use ultimate::UltimateApi as Api;

#[cfg(not(feature = "ultimate"))]
pub use mocked::MockApi as Api;

mod mocked {
    use ff_core::Result;

    pub struct MockApi {}

    impl MockApi {
        pub async fn init() -> Result<()> {
            Ok(())
        }

        pub async fn update() -> Result<()> {
            Ok(())
        }

        pub async fn close() -> Result<()> {
            Ok(())
        }
    }
}