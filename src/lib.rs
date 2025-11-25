pub mod asset_key;
pub mod asset;
pub mod equity;
pub mod future;
pub mod time_series;
pub mod sqlite_provider;
pub mod yahoo_finance;
pub mod dag;
pub mod analytics;
pub mod push_mode;
pub mod replay;
pub mod server;

#[cfg(test)]
mod integration_tests;

pub use asset_key::AssetKey;
pub use asset::{Asset, AssetType};
pub use equity::{Equity, CorporateAction, AssetMetadata};
pub use future::{Future, ExpiryCalendar};
pub use time_series::{TimeSeriesPoint, DateRange, DataProvider, DataProviderError, InMemoryDataProvider};
pub use sqlite_provider::SqliteDataProvider;
pub use yahoo_finance::{YahooFinanceDownloader, DownloaderConfig, DownloadError, DownloadResult};
pub use dag::{AnalyticsDag, Node, NodeId, NodeParams, NodeOutput, DagError};
pub use analytics::{
    calculate_returns,
    calculate_volatility,
    OutputMode,
    AnalyticsQuery,
    ReturnsQueryBuilder,
    VolatilityQueryBuilder,
    apply_output_mode,
};
pub use push_mode::{CircularBuffer, PushModeEngine, PushError, InitError, NodeState, NodePushState};
pub use replay::{ReplayEngine, ReplayError, ReplayResult};
pub use server::{run_server, ServerConfig, AppState, ApiError};

