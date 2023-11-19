use crate::sss::Polynomial;
use serde::{Deserialize, Serialize};


/// Represents a request in a simple share exchange protocol.
///
/// This enum encapsulates different types of requests that can be made, such as getting a share,
/// registering a new share, or refreshing shares.
///
/// # Variants
///
/// * `GetShare(GetShareRequest)` - Represents a request to get a share.
/// * `RegisterShare(RegisterShareRequest)` - Represents a request to register a new share.
/// * `RefreshShares(RefreshShareRequest)` - Represents a request to refresh existing shares.
///
/// # Examples
///
/// Creating a `GetShare` request:
///
/// ```rust
/// use libp2p::PeerId;
/// use mpcnet::sss::Polynomial;
/// use mpcnet::protocol::{GetShareRequest, Request};
/// 
/// let request = Request::GetShare(GetShareRequest {
///     key: "share_key".to_string(),
///     peer: vec![1, 2, 3],
///     sender: vec![4, 5, 6],
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    GetShare(GetShareRequest),
    RegisterShare(RegisterShareRequest),
    RefreshShare(RefreshShareRequest),
}

/// Represents a response in a simple share exchange protocol.
///
/// This enum encapsulates different types of responses corresponding to the requests made.
///
/// # Variants
///
/// * `GetShare(GetShareResponse)` - Response to a `GetShare` request.
/// * `RegisterShare(RegisterShareResponse)` - Response to a `RegisterShare` request.
/// * `RefreshShares(RefreshSharesResponse)` - Response to a `RefreshShares` request.
///
/// # Examples
///
/// Creating a `GetShare` response:
///
/// ```rust
/// use libp2p::PeerId;
/// use mpcnet::sss::Polynomial;
/// use mpcnet::protocol::{GetShareResponse, Response};
/// 
/// let response = Response::GetShare(GetShareResponse {
///     share: (1, vec![7, 8, 9]),
///     success: true,
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    GetShare(GetShareResponse),
    RegisterShare(RegisterShareResponse),
    RefreshShares(RefreshShareResponse),
}

/// Represents a request to get a share.
///
/// This struct is used when a client wishes to retrieve a specific share from the system.
///
/// # Fields
///
/// * `key` - A string representing the key of the share.
/// * `peer` - A byte vector representing the peer from whom the share is requested.
/// * `sender` - A byte vector representing the sender of the request.
///
/// # Examples
///
/// Creating a new `GetShareRequest`:
///
/// ```rust
/// use libp2p::PeerId;
/// use mpcnet::sss::Polynomial;
/// use mpcnet::protocol::GetShareRequest;
/// 
/// let request = GetShareRequest {
///     key: "share_key".to_string(),
///     peer: vec![1, 2, 3],
///     sender: vec![4, 5, 6],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetShareRequest {
    pub key: String,
    pub peer: Vec<u8>,
    pub sender: Vec<u8>,
}

/// Represents a response to a `GetShare` request.
///
/// This struct is used to send back the requested share along with a success status.
///
/// # Fields
///
/// * `share` - A tuple containing the share identifier (u8) and the share data (Vec<u8>).
/// * `success` - A boolean indicating whether the request was successful.
///
/// # Examples
///
/// Creating a new `GetShareResponse`:
///
/// ```rust
/// use mpcnet::protocol::GetShareResponse;
/// 
/// let response = GetShareResponse {
///     share: (1, vec![7, 8, 9]),
///     success: true,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetShareResponse {
    pub share: (u8, Vec<u8>),
    pub success: bool,
}

/// Represents a request to register a new share.
///
/// This struct is used when a client wants to add a new share to the system.
///
/// # Fields
///
/// * `key` - A string representing the key of the share.
/// * `share` - A tuple containing the share identifier (u8) and the share data (Vec<u8>).
/// * `peer` - A byte vector representing the peer with whom the share is associated.
/// * `sender` - A byte vector representing the sender of the request.
///
/// # Examples
///
/// Creating a new `RegisterShareRequest`:
///
/// ```rust
/// use mpcnet::sss::Polynomial;
/// use mpcnet::protocol::RegisterShareRequest;
/// 
/// let request = RegisterShareRequest {
///     key: "share_key".to_string(),
///     share: (1, vec![1, 2, 3]),
///     peer: vec![4, 5, 6],
///     sender: vec![7, 8, 9],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterShareRequest {
    pub key: String,
    pub share: (u8, Vec<u8>),
    pub peer: Vec<u8>,
    pub sender: Vec<u8>,
}

/// Represents a response to a `RegisterShare` request.
///
/// This struct is used to indicate the success or failure of registering a new share.
///
/// # Fields
///
/// * `success` - A boolean indicating whether the share was successfully registered.
///
/// # Examples
///
/// Creating a new `RegisterShareResponse`:
///
/// ```rust
/// use mpcnet::protocol::RegisterShareResponse;
/// 
/// let response = RegisterShareResponse {
///     success: true,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterShareResponse {
    pub success: bool,
}

/// Represents a request to refresh share.
///
/// This struct is used when a client requests to refresh the existing shares, 
/// typically to enhance their security.
///
/// # Fields
///
/// * `key` - A string representing the key associated with the share.
/// * `refresh_key` - A vector of `Polynomial` objects used in the refresh process.
/// * `peer` - A byte vector representing the peer involved in the refresh process.
/// * `sender` - A byte vector representing the sender of the request.
///
/// # Examples
///
/// Creating a new `RefreshShareRequest`:
///
/// ```rust
/// use mpcnet::sss::Polynomial;
/// use mpcnet::protocol::RefreshShareRequest;
/// use gf256::gf256;
/// 
/// let request = RefreshShareRequest {
///     key: "share_key".to_string(),
///     refresh_key: vec![Polynomial::new(2, gf256::new(5))],
///     peer: vec![1, 2, 3],
///     sender: vec![4, 5, 6],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshShareRequest {
    pub key: String,
    pub refresh_key: Vec<Polynomial>,
    pub peer: Vec<u8>,
    pub sender: Vec<u8>,
}

/// Represents a response to a `RefreshShare` request.
///
/// This struct is used to indicate the success or failure of the share refresh process.
///
/// # Fields
///
/// * `success` - A boolean indicating whether the shares were successfully refreshed.
///
/// # Examples
///
/// Creating a new `RefreshSharesResponse`:
///
/// ```rust
/// use mpcnet::protocol::RefreshSharesResponse;
/// 
/// let response = RefreshSharesResponse {
///     success: true,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshShareResponse {
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use crate::sss::Polynomial;
    use gf256::gf256;
    use libp2p::PeerId;

    use super::*;
    use cbor4ii::serde::to_vec;
    use serde::Deserialize;

    #[track_caller]
    fn de<'a, T>(bytes: &'a [u8], _value: &T) -> T
    where
        T: Deserialize<'a>,
    {
        serde_cbor::from_slice(bytes).unwrap()
    }

    macro_rules! assert_test {
        ( $value:expr ) => {{
            let buf = to_vec(Vec::new(), &$value).unwrap();
            let value = de(&buf, &$value);
            assert_eq!(value, $value);
        }};
    }

    #[test]
    fn test_serialize_deserialize_get_share_request() {
        let request = GetShareRequest {
            key: "share_id".to_string(),
            peer: PeerId::random().into(),
            sender: PeerId::random().into(),
        };
        assert_test!(request);
    }

    #[test]
    fn test_serialize_deserialize_get_share_response() {
        let response = GetShareResponse {
            share: (1u8, vec![1, 2, 3, 4]),
            success: true,
        };
        assert_test!(response);
    }

    #[test]
    fn test_serialize_deserialize_register_share_request() {
        let request = RegisterShareRequest {
            share: (1u8, vec![1, 2, 3, 4]),
            key: "unique_id".to_string(),
            peer: PeerId::random().into(),
            sender: PeerId::random().into(),
        };
        assert_test!(request);
    }

    #[test]
    fn test_serialize_deserialize_register_share_response() {
        let response = RegisterShareResponse { success: true };
        assert_test!(response);
    }

    #[test]
    fn test_serialize_deserialize_request_enum() {
        let get_share_req = Request::GetShare(GetShareRequest {
            key: "share_id".to_string(),
            peer: PeerId::random().into(),
            sender: PeerId::random().into(),
        });
        assert_test!(get_share_req);

        let register_share_req = Request::RegisterShare(RegisterShareRequest {
            share: (1u8, vec![1, 2, 3, 4]),
            key: "unique_id".to_string(),
            peer: PeerId::random().into(),
            sender: PeerId::random().into(),
        });
        assert_test!(register_share_req);
    }

    #[test]
    fn test_serialize_deserialize_response_enum() {
        let get_share_res = Response::GetShare(GetShareResponse {
            share: (1u8, vec![1, 2, 3, 4]),
            success: true,
        });
        assert_test!(get_share_res);

        let register_share_res = Response::RegisterShare(RegisterShareResponse { success: true });
        assert_test!(register_share_res);
    }

    #[test]
    fn test_serialize_deserialize_polynomial() {
        let poly = Polynomial::new(3, gf256::new(42));
        assert_test!(poly);
    }
}
