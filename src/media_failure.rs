use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaStage {
    AssetIngestion,
    ProcessingJob,
    CreatorDelivery,
}

impl MediaStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AssetIngestion => "asset_ingestion",
            Self::ProcessingJob => "processing_job",
            Self::CreatorDelivery => "creator_delivery",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaAgentError {
    InvalidAsset { asset_id: String, reason: String },
    TranscodeFailed { job_id: String, reason: String },
    DeliveryRejected { creator_id: String, reason: String },
}

impl MediaAgentError {
    pub fn stage(&self) -> MediaStage {
        match self {
            Self::InvalidAsset { .. } => MediaStage::AssetIngestion,
            Self::TranscodeFailed { .. } => MediaStage::ProcessingJob,
            Self::DeliveryRejected { .. } => MediaStage::CreatorDelivery,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidAsset { asset_id, reason } => format!("asset {asset_id}: {reason}"),
            Self::TranscodeFailed { job_id, reason } => format!("job {job_id}: {reason}"),
            Self::DeliveryRejected { creator_id, reason } => format!("creator {creator_id}: {reason}"),
        }
    }

    pub fn is_creator_actionable(&self) -> bool {
        matches!(self, Self::InvalidAsset { .. } | Self::DeliveryRejected { .. })
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FailureNotice {
    pub exception: String,
}

pub fn capture_notice(error: &MediaAgentError) -> FailureNotice {
    let action = if error.is_creator_actionable() {
        "creator_action_required"
    } else {
        "retry_processing"
    };
    FailureNotice {
        exception: format!("stage={}; action={action}; {}", error.stage().as_str(), error.message()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_asset_requests_creator_action() {
        let error = MediaAgentError::InvalidAsset {
            asset_id: "asset-42".into(),
            reason: "missing audio track".into(),
        };

        let notice = capture_notice(&error);

        assert_eq!(
            notice.exception,
            "stage=asset_ingestion; action=creator_action_required; asset asset-42: missing audio track"
        );
    }
}
