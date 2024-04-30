

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Player {
    #[prost(string, tag = "1")]
    pub email: ::prost::alloc::string::String,
    #[prost(int32, tag = "2")]
    pub id: i32,
    #[prost(string, tag = "3")]
    pub country: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub name: ::prost::alloc::string::String,
}
