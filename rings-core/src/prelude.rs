#[cfg(feature = "wasm")]
pub use web_sys::RtcSdpType as RTCSdpType;
#[cfg(not(feature = "wasm"))]
pub use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;

pub use uuid;
pub use web3;

#[cfg(feature = "wasm")]
pub use web_sys;
#[cfg(feature = "default")]
pub use webrtc;