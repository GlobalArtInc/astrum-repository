package org.globalart.astrum.repository.gitlfs.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.RedeployDisabledException;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

/**
 * Answers failed Git LFS API requests with the JSON error document clients show to their users.
 * <p>
 * It runs ahead of the security handler: a signed-in user without the privilege gets a JSON 403, while an anonymous
 * request still gets the authentication challenge git-lfs needs to ask for credentials.
 */
@Component
public class GitLfsApiErrorHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(GitLfsApiErrorHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    try {
      return asJson(context.proceed());
    }
    catch (GitLfsException e) {
      log.debug("Git LFS request {} {} failed: {}", context.getRequest().getAction(), context.getRequest().getPath(),
          e.getMessage());
      return e.getBody() != null ? GitLfsJson.response(e.getStatus(), e.getBody())
          : GitLfsJson.error(e.getStatus(), e.getMessage());
    }
    catch (AuthorizationException e) {
      if (!isAuthenticated()) {
        throw e;
      }
      return GitLfsJson.error(403, "Not permitted in repository " + context.getRepository().getName());
    }
    catch (RedeployDisabledException e) {
      return GitLfsJson.error(409, "Repository " + context.getRepository().getName() + " does not allow redeployment");
    }
    catch (IllegalOperationException e) {
      return GitLfsJson.error(403, e.getMessage());
    }
    catch (InvalidContentException | IllegalArgumentException e) {
      log.debug("Git LFS request {} {} rejected", context.getRequest().getAction(), context.getRequest().getPath(), e);
      return GitLfsJson.error(422, e.getMessage());
    }
  }

  /**
   * Turns the plain failures of the shared handlers into JSON errors.
   */
  private static Response asJson(final Response response) {
    Status status = response.getStatus();
    Payload payload = response.getPayload();
    if (status.isSuccessful() || (payload != null && GitLfsJson.MEDIA_TYPE.equals(payload.getContentType()))) {
      return response;
    }
    String message = status.getMessage() != null ? status.getMessage()
        : "Request failed with status " + status.getCode();
    return GitLfsJson.error(status.getCode(), message);
  }

  private static boolean isAuthenticated() {
    Subject subject = SecurityUtils.getSubject();
    return subject.getPrincipal() != null && subject.isAuthenticated();
  }
}
