mod base;
mod http;
mod tcp_speedtest_net;

pub use base::Client;
pub use http::HTTPClient;
pub use tcp_speedtest_net::SpeedtestNetTcpClient;
