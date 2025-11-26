pub mod analytics;
pub mod asset;
pub mod asset_key;
pub mod dag;
pub mod equity;
pub mod future;
pub mod push_mode;
pub mod replay;
pub mod server;
pub mod sqlite_provider;
pub mod time_series;
pub mod yahoo_finance;

#[cfg(test)]
mod integration_tests;

pub use analytics::{
    apply_output_mode, calculate_returns, calculate_volatility, AnalyticsQuery, OutputMode,
    ReturnsQueryBuilder, VolatilityQueryBuilder,
};
pub use asset::{Asset, AssetType};
pub use asset_key::AssetKey;
pub use dag::{AnalyticsDag, DagError, Node, NodeId, NodeOutput, NodeParams};
pub use equity::{AssetMetadata, CorporateAction, Equity};
pub use future::{ExpiryCalendar, Future};
pub use push_mode::{
    CircularBuffer, InitError, NodePushState, NodeState, PushError, PushModeEngine,
};
pub use replay::{ReplayEngine, ReplayError, ReplayResult};
pub use server::{run_server, ApiError, AppState, ServerConfig};
pub use sqlite_provider::SqliteDataProvider;
pub use time_series::{
    DataProvider, DataProviderError, DateRange, InMemoryDataProvider, TimeSeriesPoint,
};
pub use yahoo_finance::{DownloadError, DownloadResult, DownloaderConfig, YahooFinanceDownloader};
