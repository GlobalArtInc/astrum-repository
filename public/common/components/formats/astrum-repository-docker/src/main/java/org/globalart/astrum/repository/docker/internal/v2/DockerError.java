package org.globalart.astrum.repository.docker.internal.v2;

public enum DockerError
{
  BLOB_UNKNOWN(404, "blob unknown to registry"),
  BLOB_UPLOAD_INVALID(400, "blob upload invalid"),
  BLOB_UPLOAD_UNKNOWN(404, "blob upload unknown to registry"),
  DIGEST_INVALID(400, "provided digest did not match uploaded content"),
  MANIFEST_BLOB_UNKNOWN(400, "blob unknown to registry"),
  MANIFEST_INVALID(400, "manifest invalid"),
  MANIFEST_UNKNOWN(404, "manifest unknown"),
  NAME_INVALID(400, "invalid repository name"),
  NAME_UNKNOWN(404, "repository name not known to registry"),
  SIZE_INVALID(400, "provided length did not match content length"),
  RANGE_INVALID(416, "invalid content range"),
  UNAUTHORIZED(401, "authentication required"),
  DENIED(403, "requested access to the resource is denied"),
  UNSUPPORTED(405, "the operation is unsupported"),
  TOOMANYREQUESTS(429, "too many requests"),
  UNKNOWN(500, "unknown error");

  private final int status;

  private final String message;

  DockerError(final int status, final String message) {
    this.status = status;
    this.message = message;
  }

  public int status() {
    return status;
  }

  public String message() {
    return message;
  }
}
