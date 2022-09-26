// @generated
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetModeRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetModeResponse {
    #[prost(enumeration="Mode", tag="1")]
    pub mode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetModeRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration="Mode", tag="2")]
    pub mode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetModeResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Waypoint {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub location: ::core::option::Option<super::super::super::common::v1::GeoPoint>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLocationRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLocationResponse {
    #[prost(message, optional, tag="1")]
    pub location: ::core::option::Option<super::super::super::common::v1::GeoPoint>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetWaypointsRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetWaypointsResponse {
    #[prost(message, repeated, tag="1")]
    pub waypoints: ::prost::alloc::vec::Vec<Waypoint>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddWaypointRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub location: ::core::option::Option<super::super::super::common::v1::GeoPoint>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddWaypointResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveWaypointRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveWaypointResponse {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Mode {
    Unspecified = 0,
    Manual = 1,
    Waypoint = 2,
}
// @@protoc_insertion_point(module)