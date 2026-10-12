package org.globalart.astrum.repository.gitlfs.internal;

import javax.annotation.Nullable;

/**
 * A request the Git LFS API answers with an error status and a JSON message.
 */
public class GitLfsException
    extends RuntimeException
{
  private final int status;

  @Nullable
  private final transient Object body;

  public GitLfsException(final int status, final String message) {
    this(status, message, null);
  }

  /**
   * @param body the response body to send instead of the plain message, as the lock API does on conflicts
   */
  public GitLfsException(final int status, final String message, @Nullable final Object body) {
    super(message);
    this.status = status;
    this.body = body;
  }

  public int getStatus() {
    return status;
  }

  @Nullable
  public Object getBody() {
    return body;
  }

  public static GitLfsException badRequest(final String message) {
    return new GitLfsException(400, message);
  }

  public static GitLfsException forbidden(final String message) {
    return new GitLfsException(403, message);
  }

  public static GitLfsException notFound(final String message) {
    return new GitLfsException(404, message);
  }

  public static GitLfsException unprocessable(final String message) {
    return new GitLfsException(422, message);
  }
}
