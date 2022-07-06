//pub mod benchmark;
pub mod gw_node_connection;
pub mod latest_block;
pub mod random_block;

//pub use benchmark::generator::BenchmarkGenerator;
pub use gw_node_connection::TaskGWNodeConnection;
pub use latest_block::generator::LatestBlockGenerator;
pub use random_block::generator::TaskRandomBlock;
