// @generated
/// this is an experimental API message
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrameSystemConfig {
    #[prost(message, optional, tag="1")]
    pub frame: ::core::option::Option<super::super::common::v1::Transform>,
    #[prost(message, optional, tag="2")]
    pub kinematics: ::core::option::Option<::prost_types::Struct>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrameSystemConfigRequest {
    /// pose information on any additional reference frames that are needed
    /// to supplement the robot's frame system
    #[prost(message, repeated, tag="1")]
    pub supplemental_transforms: ::prost::alloc::vec::Vec<super::super::common::v1::Transform>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrameSystemConfigResponse {
    #[prost(message, repeated, tag="1")]
    pub frame_system_configs: ::prost::alloc::vec::Vec<FrameSystemConfig>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransformPoseRequest {
    /// the original pose to transform along with the reference frame in
    /// which it was observed
    #[prost(message, optional, tag="1")]
    pub source: ::core::option::Option<super::super::common::v1::PoseInFrame>,
    /// the reference frame into which the source pose should be transformed,
    /// if unset this defaults to the "world" reference frame
    #[prost(string, tag="2")]
    pub destination: ::prost::alloc::string::String,
    /// pose information on any additional reference frames that are needed
    /// to perform the transform
    #[prost(message, repeated, tag="3")]
    pub supplemental_transforms: ::prost::alloc::vec::Vec<super::super::common::v1::Transform>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransformPoseResponse {
    #[prost(message, optional, tag="1")]
    pub pose: ::core::option::Option<super::super::common::v1::PoseInFrame>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceNamesRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceNamesResponse {
    #[prost(message, repeated, tag="1")]
    pub resources: ::prost::alloc::vec::Vec<super::super::common::v1::ResourceName>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceRpcSubtype {
    #[prost(message, optional, tag="1")]
    pub subtype: ::core::option::Option<super::super::common::v1::ResourceName>,
    #[prost(string, tag="2")]
    pub proto_service: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceRpcSubtypesRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceRpcSubtypesResponse {
    #[prost(message, repeated, tag="1")]
    pub resource_rpc_subtypes: ::prost::alloc::vec::Vec<ResourceRpcSubtype>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Operation {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub method: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub arguments: ::core::option::Option<::prost_types::Struct>,
    #[prost(message, optional, tag="4")]
    pub started: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, optional, tag="5")]
    pub session_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOperationsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOperationsResponse {
    #[prost(message, repeated, tag="1")]
    pub operations: ::prost::alloc::vec::Vec<Operation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOperationRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOperationResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockForOperationRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockForOperationResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeerConnectionInfo {
    #[prost(enumeration="PeerConnectionType", tag="1")]
    pub r#type: i32,
    #[prost(string, optional, tag="2")]
    pub remote_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub local_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Session {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub peer_connection_info: ::core::option::Option<PeerConnectionInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSessionsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSessionsResponse {
    #[prost(message, repeated, tag="1")]
    pub sessions: ::prost::alloc::vec::Vec<Session>,
}
// Discovery

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscoveryQuery {
    #[prost(string, tag="1")]
    pub subtype: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub model: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Discovery {
    #[prost(message, optional, tag="1")]
    pub query: ::core::option::Option<DiscoveryQuery>,
    #[prost(message, optional, tag="2")]
    pub results: ::core::option::Option<::prost_types::Struct>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscoverComponentsRequest {
    #[prost(message, repeated, tag="1")]
    pub queries: ::prost::alloc::vec::Vec<DiscoveryQuery>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscoverComponentsResponse {
    #[prost(message, repeated, tag="1")]
    pub discovery: ::prost::alloc::vec::Vec<Discovery>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Status {
    #[prost(message, optional, tag="1")]
    pub name: ::core::option::Option<super::super::common::v1::ResourceName>,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<::prost_types::Struct>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusRequest {
    #[prost(message, repeated, tag="1")]
    pub resource_names: ::prost::alloc::vec::Vec<super::super::common::v1::ResourceName>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusResponse {
    #[prost(message, repeated, tag="1")]
    pub status: ::prost::alloc::vec::Vec<Status>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamStatusRequest {
    #[prost(message, repeated, tag="1")]
    pub resource_names: ::prost::alloc::vec::Vec<super::super::common::v1::ResourceName>,
    /// how often to send a new status.
    #[prost(message, optional, tag="2")]
    pub every: ::core::option::Option<::prost_types::Duration>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamStatusResponse {
    #[prost(message, repeated, tag="1")]
    pub status: ::prost::alloc::vec::Vec<Status>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopExtraParameters {
    #[prost(message, optional, tag="1")]
    pub name: ::core::option::Option<super::super::common::v1::ResourceName>,
    #[prost(message, optional, tag="2")]
    pub params: ::core::option::Option<::prost_types::Struct>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopAllRequest {
    #[prost(message, repeated, tag="99")]
    pub extra: ::prost::alloc::vec::Vec<StopExtraParameters>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopAllResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartSessionRequest {
    /// resume can be used to attempt to continue a stream after a disconnection event. If
    /// a session is not found, a new one will be created and returned.
    #[prost(string, tag="1")]
    pub resume: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartSessionResponse {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub heartbeat_window: ::core::option::Option<::prost_types::Duration>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendSessionHeartbeatRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendSessionHeartbeatResponse {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PeerConnectionType {
    Unspecified = 0,
    Grpc = 1,
    Webrtc = 2,
}
impl PeerConnectionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PeerConnectionType::Unspecified => "PEER_CONNECTION_TYPE_UNSPECIFIED",
            PeerConnectionType::Grpc => "PEER_CONNECTION_TYPE_GRPC",
            PeerConnectionType::Webrtc => "PEER_CONNECTION_TYPE_WEBRTC",
        }
    }
}
// @@protoc_insertion_point(module)
