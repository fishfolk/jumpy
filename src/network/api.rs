#[cfg(not(feature = "macroquad"))]
// pub use ultimate::UltimateApi as Api;
#[cfg(feature = "macroquad")]
pub use mocked::MockApi as Api;

#[allow(dead_code)]
mod mocked {
    use ff_core::result::Result;

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
