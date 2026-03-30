
package org.globalart.astrum.repository.npm.internal;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.ContentTypes;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.google.common.collect.Maps;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpStatus.BAD_REQUEST;
import static org.globalart.astrum.repository.http.HttpStatus.FORBIDDEN;
import static org.globalart.astrum.repository.http.HttpStatus.NOT_FOUND;
import static org.globalart.astrum.repository.http.HttpStatus.UNAUTHORIZED;

/**
 * npm response utility, that sends status messages along with JSON, as npm CLI expects.
 *
 * @since 3.0
 */
public final class NpmResponses
{
  private NpmResponses() {
    // nop
  }

  private static final Payload successPayload = statusPayload(true, null);

  private static Payload statusPayload(final boolean success, @Nullable final String error) {
    NestedAttributesMap errorObject = new NestedAttributesMap("error", Maps.newHashMap());
    errorObject.set("success", success);
    if (error != null) {
      errorObject.set("error", error);
    }
    return new BytesPayload(NpmJsonUtils.bytes(errorObject), ContentTypes.APPLICATION_JSON);
  }

  @Nonnull
  public static Response ok() {
    return HttpResponses.ok(successPayload);
  }

  @Nonnull
  public static Response ok(@Nonnull final Payload payload) {
    return HttpResponses.ok(checkNotNull(payload));
  }

  @Nonnull
  public static Response notFound(@Nullable final String message) {
    return failureWithStatusPayload(NOT_FOUND, message);
  }

  @Nonnull
  public static Response packageNotFound(final NpmPackageId packageId) {
    checkNotNull(packageId);
    return notFound("Package '" + packageId + "' not found");
  }

  @Nonnull
  public static Response tarballNotFound(final NpmPackageId packageId, final String tarballName) {
    checkNotNull(packageId);
    checkNotNull(tarballName);
    return notFound("Tarball '" + tarballName + "' in package '" + packageId + "' not found");
  }

  @Nonnull
  public static Response badRequest(@Nullable final String message) {
    return failureWithStatusPayload(BAD_REQUEST, message);
  }

  @Nonnull
  public static Response badCredentials(@Nullable final String message) {
    return failureWithStatusPayload(UNAUTHORIZED, message);
  }

  @Nonnull
  public static Response forbidden(@Nullable final String message) {
    return failureWithStatusPayload(FORBIDDEN, message);
  }

  @Nonnull
  public static Response failureWithStatusPayload(final int code, @Nullable final String message) {
    return new Response.Builder()
        .status(Status.failure(code))
        .payload(statusPayload(false, message))
        .build();
  }
}
