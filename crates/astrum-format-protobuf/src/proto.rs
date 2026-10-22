use prost::{Enumeration, Message};

#[derive(Clone, Copy, PartialEq, Eq, Message)]
pub struct Timestamp {
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum DigestType {
    Unspecified = 0,
    B5 = 1,
}

#[derive(Clone, PartialEq, Message)]
pub struct Digest {
    #[prost(enumeration = "DigestType", tag = "1")]
    pub kind: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub value: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Commit {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub create_time: Option<Timestamp>,
    #[prost(string, tag = "3")]
    pub owner_id: String,
    #[prost(string, tag = "4")]
    pub module_id: String,
    #[prost(message, optional, tag = "5")]
    pub digest: Option<Digest>,
    #[prost(string, tag = "6")]
    pub created_by_user_id: String,
    #[prost(string, tag = "7")]
    pub source_control_url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum ModuleVisibility {
    Unspecified = 0,
    Public = 1,
    Private = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum ModuleState {
    Unspecified = 0,
    Active = 1,
    Deprecated = 2,
}

#[derive(Clone, PartialEq, Message)]
pub struct Module {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub create_time: Option<Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub update_time: Option<Timestamp>,
    #[prost(string, tag = "4")]
    pub name: String,
    #[prost(string, tag = "5")]
    pub owner_id: String,
    #[prost(enumeration = "ModuleVisibility", tag = "6")]
    pub visibility: i32,
    #[prost(enumeration = "ModuleState", tag = "7")]
    pub state: i32,
    #[prost(string, tag = "8")]
    pub description: String,
    #[prost(string, tag = "9")]
    pub url: String,
    #[prost(string, tag = "10")]
    pub default_label_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct ModuleRef {
    #[prost(oneof = "module_ref::Value", tags = "1, 2")]
    pub value: Option<module_ref::Value>,
}

pub mod module_ref {
    use prost::{Message, Oneof};

    #[derive(Clone, PartialEq, Message)]
    pub struct Name {
        #[prost(string, tag = "1")]
        pub owner: String,
        #[prost(string, tag = "2")]
        pub module: String,
    }

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        Id(String),
        #[prost(message, tag = "2")]
        Name(Name),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum CommitCheckStatus {
    Unspecified = 0,
    Disabled = 1,
    Passed = 2,
    Pending = 3,
    Rejected = 4,
    Approved = 5,
}

#[derive(Clone, PartialEq, Message)]
pub struct CommitCheckState {
    #[prost(enumeration = "CommitCheckStatus", tag = "1")]
    pub status: i32,
    #[prost(message, optional, tag = "3")]
    pub update_time: Option<Timestamp>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Label {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub create_time: Option<Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub update_time: Option<Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub archive_time: Option<Timestamp>,
    #[prost(string, tag = "5")]
    pub name: String,
    #[prost(string, tag = "6")]
    pub owner_id: String,
    #[prost(string, tag = "7")]
    pub module_id: String,
    #[prost(string, tag = "8")]
    pub commit_id: String,
    #[prost(string, tag = "9")]
    pub updated_by_user_id: String,
    #[prost(message, optional, tag = "10")]
    pub commit_check_state: Option<CommitCheckState>,
}

#[derive(Clone, PartialEq, Message)]
pub struct LabelRef {
    #[prost(oneof = "label_ref::Value", tags = "1, 2")]
    pub value: Option<label_ref::Value>,
}

pub mod label_ref {
    use prost::{Message, Oneof};

    #[derive(Clone, PartialEq, Message)]
    pub struct Name {
        #[prost(string, tag = "1")]
        pub owner: String,
        #[prost(string, tag = "2")]
        pub module: String,
        #[prost(string, tag = "3")]
        pub label: String,
    }

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        Id(String),
        #[prost(message, tag = "2")]
        Name(Name),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ScopedLabelRef {
    #[prost(oneof = "scoped_label_ref::Value", tags = "1, 2")]
    pub value: Option<scoped_label_ref::Value>,
}

pub mod scoped_label_ref {
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        Id(String),
        #[prost(string, tag = "2")]
        Name(String),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ResourceRef {
    #[prost(oneof = "resource_ref::Value", tags = "1, 2")]
    pub value: Option<resource_ref::Value>,
}

pub mod resource_ref {
    use prost::{Message, Oneof};

    #[derive(Clone, PartialEq, Message)]
    pub struct Name {
        #[prost(string, tag = "1")]
        pub owner: String,
        #[prost(string, tag = "2")]
        pub module: String,
        #[prost(oneof = "name::Child", tags = "3, 4")]
        pub child: Option<name::Child>,
    }

    pub mod name {
        use prost::Oneof;

        #[derive(Clone, PartialEq, Oneof)]
        pub enum Child {
            #[prost(string, tag = "3")]
            LabelName(String),
            #[prost(string, tag = "4")]
            Ref(String),
        }
    }

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        Id(String),
        #[prost(message, tag = "2")]
        Name(Name),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Resource {
    #[prost(oneof = "resource::Value", tags = "1, 2, 3")]
    pub value: Option<resource::Value>,
}

pub mod resource {
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Module(super::Module),
        #[prost(message, tag = "2")]
        Label(super::Label),
        #[prost(message, tag = "3")]
        Commit(super::Commit),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct File {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(bytes = "vec", tag = "2")]
    pub content: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum FileType {
    Unspecified = 0,
    Proto = 1,
    Doc = 2,
    License = 3,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetModulesRequest {
    #[prost(message, repeated, tag = "1")]
    pub module_refs: Vec<ModuleRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetModulesResponse {
    #[prost(message, repeated, tag = "1")]
    pub modules: Vec<Module>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ListModulesRequest {
    #[prost(uint32, tag = "1")]
    pub page_size: u32,
    #[prost(string, tag = "2")]
    pub page_token: String,
    #[prost(message, repeated, tag = "3")]
    pub owner_refs: Vec<OwnerRef>,
    #[prost(enumeration = "list_modules_request::Order", tag = "4")]
    pub order: i32,
}

pub mod list_modules_request {
    use prost::Enumeration;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    #[repr(i32)]
    pub enum Order {
        Unspecified = 0,
        CreateTimeDesc = 1,
        CreateTimeAsc = 2,
        UpdateTimeDesc = 3,
        UpdateTimeAsc = 4,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ListModulesResponse {
    #[prost(string, tag = "1")]
    pub next_page_token: String,
    #[prost(message, repeated, tag = "2")]
    pub modules: Vec<Module>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CreateModulesRequest {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<create_modules_request::Value>,
}

pub mod create_modules_request {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(message, optional, tag = "1")]
        pub owner_ref: Option<super::OwnerRef>,
        #[prost(string, tag = "2")]
        pub name: String,
        #[prost(enumeration = "super::ModuleVisibility", tag = "3")]
        pub visibility: i32,
        #[prost(string, tag = "4")]
        pub description: String,
        #[prost(string, tag = "5")]
        pub url: String,
        #[prost(string, tag = "6")]
        pub default_label_name: String,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CreateModulesResponse {
    #[prost(message, repeated, tag = "1")]
    pub modules: Vec<Module>,
}

#[derive(Clone, PartialEq, Message)]
pub struct UpdateModulesRequest {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<update_modules_request::Value>,
}

pub mod update_modules_request {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(message, optional, tag = "1")]
        pub module_ref: Option<super::ModuleRef>,
        #[prost(enumeration = "super::ModuleVisibility", optional, tag = "3")]
        pub visibility: Option<i32>,
        #[prost(enumeration = "super::ModuleState", optional, tag = "4")]
        pub state: Option<i32>,
        #[prost(string, optional, tag = "5")]
        pub description: Option<String>,
        #[prost(string, optional, tag = "6")]
        pub url: Option<String>,
        #[prost(string, optional, tag = "7")]
        pub default_label_name: Option<String>,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct UpdateModulesResponse {
    #[prost(message, repeated, tag = "1")]
    pub modules: Vec<Module>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DeleteModulesRequest {
    #[prost(message, repeated, tag = "1")]
    pub module_refs: Vec<ModuleRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DeleteModulesResponse {}

#[derive(Clone, PartialEq, Message)]
pub struct UploadRequest {
    #[prost(message, repeated, tag = "1")]
    pub contents: Vec<upload_request::Content>,
    #[prost(string, repeated, tag = "2")]
    pub dep_commit_ids: Vec<String>,
}

pub mod upload_request {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Content {
        #[prost(message, optional, tag = "1")]
        pub module_ref: Option<super::ModuleRef>,
        #[prost(message, repeated, tag = "2")]
        pub files: Vec<super::File>,
        #[prost(message, repeated, tag = "3")]
        pub scoped_label_refs: Vec<super::ScopedLabelRef>,
        #[prost(string, tag = "4")]
        pub source_control_url: String,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct UploadResponse {
    #[prost(message, repeated, tag = "1")]
    pub commits: Vec<Commit>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetCommitsRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_refs: Vec<ResourceRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetCommitsResponse {
    #[prost(message, repeated, tag = "1")]
    pub commits: Vec<Commit>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ListCommitsRequest {
    #[prost(uint32, tag = "1")]
    pub page_size: u32,
    #[prost(string, tag = "2")]
    pub page_token: String,
    #[prost(message, optional, tag = "3")]
    pub resource_ref: Option<ResourceRef>,
    #[prost(enumeration = "list_commits_request::Order", tag = "4")]
    pub order: i32,
    #[prost(string, tag = "5")]
    pub id_query: String,
}

pub mod list_commits_request {
    use prost::Enumeration;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    #[repr(i32)]
    pub enum Order {
        Unspecified = 0,
        CreateTimeDesc = 1,
        CreateTimeAsc = 2,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ListCommitsResponse {
    #[prost(string, tag = "1")]
    pub next_page_token: String,
    #[prost(message, repeated, tag = "2")]
    pub commits: Vec<Commit>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Graph {
    #[prost(message, repeated, tag = "1")]
    pub commits: Vec<Commit>,
    #[prost(message, repeated, tag = "2")]
    pub edges: Vec<graph::Edge>,
    #[prost(message, repeated, tag = "3")]
    pub registry_commit_ids: Vec<graph::RegistryCommitId>,
}

pub mod graph {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Node {
        #[prost(string, tag = "1")]
        pub commit_id: String,
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Edge {
        #[prost(message, optional, tag = "1")]
        pub from_node: Option<Node>,
        #[prost(message, optional, tag = "2")]
        pub to_node: Option<Node>,
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct RegistryCommitId {
        #[prost(string, tag = "1")]
        pub commit_id: String,
        #[prost(string, tag = "2")]
        pub registry: String,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct GetGraphRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_refs: Vec<ResourceRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetGraphResponse {
    #[prost(message, optional, tag = "1")]
    pub graph: Option<Graph>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DownloadRequest {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<download_request::Value>,
}

pub mod download_request {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(message, optional, tag = "1")]
        pub resource_ref: Option<super::ResourceRef>,
        #[prost(enumeration = "super::FileType", repeated, packed = "true", tag = "2")]
        pub file_types: Vec<i32>,
        #[prost(string, repeated, tag = "3")]
        pub paths: Vec<String>,
        #[prost(bool, tag = "4")]
        pub paths_allow_not_exist: bool,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct DownloadResponse {
    #[prost(message, repeated, tag = "1")]
    pub contents: Vec<download_response::Content>,
}

pub mod download_response {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Content {
        #[prost(message, optional, tag = "1")]
        pub commit: Option<super::Commit>,
        #[prost(message, repeated, tag = "2")]
        pub files: Vec<super::File>,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct GetLabelsRequest {
    #[prost(message, repeated, tag = "1")]
    pub label_refs: Vec<LabelRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetLabelsResponse {
    #[prost(message, repeated, tag = "1")]
    pub labels: Vec<Label>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ListLabelsRequest {
    #[prost(uint32, tag = "1")]
    pub page_size: u32,
    #[prost(string, tag = "2")]
    pub page_token: String,
    #[prost(message, optional, tag = "3")]
    pub resource_ref: Option<ResourceRef>,
    #[prost(enumeration = "list_labels_request::Order", tag = "4")]
    pub order: i32,
    #[prost(
        enumeration = "CommitCheckStatus",
        repeated,
        packed = "true",
        tag = "5"
    )]
    pub commit_check_statuses: Vec<i32>,
    #[prost(string, tag = "6")]
    pub name_query: String,
    #[prost(enumeration = "list_labels_request::ArchiveFilter", tag = "7")]
    pub archive_filter: i32,
}

pub mod list_labels_request {
    use prost::Enumeration;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    #[repr(i32)]
    pub enum Order {
        Unspecified = 0,
        CreateTimeDesc = 1,
        CreateTimeAsc = 2,
        UpdateTimeDesc = 3,
        UpdateTimeAsc = 4,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    #[repr(i32)]
    pub enum ArchiveFilter {
        Unspecified = 0,
        UnarchivedOnly = 1,
        ArchivedOnly = 2,
        All = 3,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ListLabelsResponse {
    #[prost(string, tag = "1")]
    pub next_page_token: String,
    #[prost(message, repeated, tag = "2")]
    pub labels: Vec<Label>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ListLabelHistoryRequest {
    #[prost(uint32, tag = "1")]
    pub page_size: u32,
    #[prost(string, tag = "2")]
    pub page_token: String,
    #[prost(message, optional, tag = "3")]
    pub label_ref: Option<LabelRef>,
    #[prost(enumeration = "list_label_history_request::Order", tag = "4")]
    pub order: i32,
    #[prost(
        enumeration = "CommitCheckStatus",
        repeated,
        packed = "true",
        tag = "5"
    )]
    pub commit_check_statuses: Vec<i32>,
    #[prost(string, tag = "6")]
    pub start_commit_id: String,
    #[prost(bool, tag = "7")]
    pub only_commits_with_changed_digests: bool,
}

pub mod list_label_history_request {
    use prost::Enumeration;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    #[repr(i32)]
    pub enum Order {
        Unspecified = 0,
        Desc = 1,
        Asc = 2,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ListLabelHistoryResponse {
    #[prost(string, tag = "1")]
    pub next_page_token: String,
    #[prost(message, repeated, tag = "2")]
    pub values: Vec<list_label_history_response::Value>,
}

pub mod list_label_history_response {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(message, optional, tag = "1")]
        pub commit: Option<super::Commit>,
        #[prost(message, optional, tag = "2")]
        pub commit_check_state: Option<super::CommitCheckState>,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CreateOrUpdateLabelsRequest {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<create_or_update_labels_request::Value>,
}

pub mod create_or_update_labels_request {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(message, optional, tag = "1")]
        pub label_ref: Option<super::LabelRef>,
        #[prost(string, tag = "2")]
        pub commit_id: String,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CreateOrUpdateLabelsResponse {
    #[prost(message, repeated, tag = "1")]
    pub labels: Vec<Label>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ArchiveLabelsRequest {
    #[prost(message, repeated, tag = "1")]
    pub label_refs: Vec<LabelRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ArchiveLabelsResponse {}

#[derive(Clone, PartialEq, Message)]
pub struct UnarchiveLabelsRequest {
    #[prost(message, repeated, tag = "1")]
    pub label_refs: Vec<LabelRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct UnarchiveLabelsResponse {}

#[derive(Clone, PartialEq, Message)]
pub struct GetResourcesRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_refs: Vec<ResourceRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetResourcesResponse {
    #[prost(message, repeated, tag = "1")]
    pub resources: Vec<Resource>,
}

#[derive(Clone, PartialEq, Message)]
pub struct OwnerRef {
    #[prost(oneof = "owner_ref::Value", tags = "1, 2")]
    pub value: Option<owner_ref::Value>,
}

pub mod owner_ref {
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        Id(String),
        #[prost(string, tag = "2")]
        Name(String),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum OrganizationVerificationStatus {
    Unspecified = 0,
    Unverified = 1,
    Verified = 2,
    Official = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum OrganizationVisibility {
    Unspecified = 0,
    Public = 1,
    Private = 2,
}

#[derive(Clone, PartialEq, Message)]
pub struct Organization {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub create_time: Option<Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub update_time: Option<Timestamp>,
    #[prost(string, tag = "4")]
    pub name: String,
    #[prost(string, tag = "5")]
    pub description: String,
    #[prost(string, tag = "6")]
    pub url: String,
    #[prost(enumeration = "OrganizationVerificationStatus", tag = "7")]
    pub verification_status: i32,
    #[prost(enumeration = "OrganizationVisibility", tag = "8")]
    pub visibility: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Owner {
    #[prost(oneof = "owner::Value", tags = "2")]
    pub value: Option<owner::Value>,
}

pub mod owner {
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(message, tag = "2")]
        Organization(super::Organization),
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct GetOwnersRequest {
    #[prost(message, repeated, tag = "1")]
    pub owner_refs: Vec<OwnerRef>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetOwnersResponse {
    #[prost(message, repeated, tag = "1")]
    pub owners: Vec<Owner>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum UserType {
    Unspecified = 0,
    Personal = 1,
    Machine = 2,
    System = 3,
}

#[derive(Clone, PartialEq, Message)]
pub struct CurrentUser {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub create_time: Option<Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub update_time: Option<Timestamp>,
    #[prost(string, tag = "4")]
    pub username: String,
    #[prost(bool, tag = "5")]
    pub deactivated: bool,
    #[prost(enumeration = "UserType", tag = "9")]
    pub user_type: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetCurrentUserRequest {}

#[derive(Clone, PartialEq, Message)]
pub struct GetCurrentUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: Option<CurrentUser>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetCurrentUserSubjectRequest {}

#[derive(Clone, PartialEq, Message)]
pub struct GetCurrentUserSubjectResponse {
    #[prost(string, tag = "1")]
    pub subject: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resource_ref_by_name_decodes_from_the_wire_form() {
        let wire = [
            0x12, 0x15, 0x0a, 0x04, b'a', b'c', b'm', b'e', 0x12, 0x07, b'g', b'r', b'e', b'e',
            b't', b'e', b'r', 0x1a, 0x04, b'm', b'a', b'i', b'n',
        ];
        let reference = ResourceRef::decode(wire.as_slice()).unwrap();
        assert_eq!(
            reference.value,
            Some(resource_ref::Value::Name(resource_ref::Name {
                owner: "acme".to_owned(),
                module: "greeter".to_owned(),
                child: Some(resource_ref::name::Child::LabelName("main".to_owned())),
            }))
        );
        assert_eq!(reference.encode_to_vec(), wire);
    }

    #[test]
    fn a_commit_encodes_the_fields_of_the_public_definition() {
        let commit = Commit {
            id: "ab".to_owned(),
            create_time: Some(Timestamp {
                seconds: 5,
                nanos: 7,
            }),
            owner_id: "o".to_owned(),
            module_id: "m".to_owned(),
            digest: Some(Digest {
                kind: DigestType::B5 as i32,
                value: vec![1, 2],
            }),
            created_by_user_id: String::new(),
            source_control_url: "u".to_owned(),
        };
        let wire = commit.encode_to_vec();
        assert_eq!(
            wire,
            vec![
                0x0a, 0x02, b'a', b'b', 0x12, 0x04, 0x08, 0x05, 0x10, 0x07, 0x1a, 0x01, b'o', 0x22,
                0x01, b'm', 0x2a, 0x06, 0x08, 0x01, 0x12, 0x02, 0x01, 0x02, 0x3a, 0x01, b'u'
            ]
        );
        assert_eq!(Commit::decode(wire.as_slice()).unwrap(), commit);
    }

    #[test]
    fn optional_module_updates_tell_an_empty_value_from_no_value() {
        let update = update_modules_request::Value {
            module_ref: None,
            visibility: None,
            state: None,
            description: Some(String::new()),
            url: None,
            default_label_name: None,
        };
        let wire = update.encode_to_vec();
        assert_eq!(wire, vec![0x2a, 0x00]);
        let decoded = update_modules_request::Value::decode(wire.as_slice()).unwrap();
        assert_eq!(decoded.description, Some(String::new()));
        assert_eq!(decoded.url, None);
    }

    #[test]
    fn the_current_user_has_its_name_in_field_four() {
        let user = CurrentUser {
            id: String::new(),
            create_time: None,
            update_time: None,
            username: "ci".to_owned(),
            deactivated: false,
            user_type: UserType::Personal as i32,
        };
        let response = GetCurrentUserResponse { user: Some(user) };
        assert_eq!(
            response.encode_to_vec(),
            vec![0x0a, 0x06, 0x22, 0x02, b'c', b'i', 0x48, 0x01]
        );
    }

    #[test]
    fn download_file_types_are_packed() {
        let value = download_request::Value {
            resource_ref: None,
            file_types: vec![FileType::Proto as i32, FileType::License as i32],
            paths: vec![],
            paths_allow_not_exist: true,
        };
        let wire = value.encode_to_vec();
        assert_eq!(wire, vec![0x12, 0x02, 0x01, 0x03, 0x20, 0x01]);
        assert_eq!(
            download_request::Value::decode([0x10, 0x01, 0x10, 0x02].as_slice())
                .unwrap()
                .file_types,
            vec![1, 2]
        );
    }
}
