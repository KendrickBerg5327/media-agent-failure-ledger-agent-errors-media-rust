mod infrai_errors;
mod media_failure;

use infrai_errors::{InfraiError, InfraiErrors};
use media_failure::{capture_notice, MediaAgentError};

async fn process_creator_asset(asset_id: &str) -> Result<(), MediaAgentError> {
    Err(MediaAgentError::InvalidAsset {
        asset_id: asset_id.to_owned(),
        reason: "missing audio track".to_owned(),
    })
}

#[tokio::main]
async fn main() {
    let error = process_creator_asset("asset-42").await.expect_err("the sample models an invalid incoming asset");
    let reporter = match InfraiErrors::from_environment() {
        Ok(reporter) => reporter,
        Err(InfraiError::MissingApiKey) => {
            eprintln!("set INFRAI_API_KEY before running this example");
            return;
        }
        Err(other) => {
            eprintln!("reporter setup failed: {other:?}");
            return;
        }
    };

    match reporter.capture(&capture_notice(&error)).await {
        Ok(()) => println!("captured asset-42 for creator follow-up"),
        Err(InfraiError::Rejected { status, error }) => eprintln!("capture rejected ({status}): {error}"),
        Err(other) => eprintln!("capture could not be completed: {other:?}"),
    }
}
