use astrum_content::Ctx;
use sha2::{Digest as _, Sha256};

use crate::connect::{Code, Reply, Status};
use crate::proto;

fn signed_in(ctx: &Ctx) -> Reply<&str> {
    match ctx.subject.as_ref() {
        Some(subject) if !subject.anonymous => Ok(subject.user_id.as_str()),
        _ => Err(Status::new(
            Code::Unauthenticated,
            "authentication is required",
        )),
    }
}

pub fn user_id(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"astrum.protobuf.user\0");
    hasher.update(name.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}

pub async fn current_user(
    ctx: &Ctx,
    _request: proto::GetCurrentUserRequest,
) -> Reply<proto::GetCurrentUserResponse> {
    let name = signed_in(ctx)?;
    Ok(proto::GetCurrentUserResponse {
        user: Some(proto::CurrentUser {
            id: user_id(name),
            create_time: None,
            update_time: None,
            username: name.to_owned(),
            deactivated: false,
            user_type: proto::UserType::Personal as i32,
        }),
    })
}

pub async fn current_subject(
    ctx: &Ctx,
    _request: proto::GetCurrentUserSubjectRequest,
) -> Reply<proto::GetCurrentUserSubjectResponse> {
    Ok(proto::GetCurrentUserSubjectResponse {
        subject: signed_in(ctx)?.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_user_id_is_thirty_two_hex_characters_that_depend_on_the_name() {
        assert_eq!(user_id("admin").len(), 32);
        assert_eq!(user_id("admin"), user_id("admin"));
        assert_ne!(user_id("admin"), user_id("ci"));
    }
}
