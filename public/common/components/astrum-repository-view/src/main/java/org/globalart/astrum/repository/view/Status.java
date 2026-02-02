
package org.globalart.astrum.repository.view;

import java.io.Serializable;

import javax.annotation.Nullable;

// TODO: implement Externalizable?  This is made Serialzable just for cache usage, may need to revisit to avoid needing
// TODO: ... to have the cache semantics leak into this core api.  Negative-cache may want to wrap or we may want to
// TODO: ... consider an interface instead, though that has non-trivial impact to code-base atm?

/**
 * A representation of response status with an optional message.
 *
 * @since 3.0
 */
public class Status
    implements Serializable
{
  private final boolean successful;

  private final int code;

  private final String message;

  public Status(final boolean successful, final int code, @Nullable final String message) {
    this.successful = successful;
    this.code = code;
    this.message = message;
  }

  public Status(final boolean successful, final int code) {
    this(successful, code, null);
  }

  public boolean isSuccessful() {
    return successful;
  }

  public int getCode() {
    return code;
  }

  @Nullable
  public String getMessage() {
    return message;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "successful=" + successful +
        ", code=" + code +
        ", message='" + message + '\'' +
        '}';
  }

  //
  // Helpers
  //

  public static Status success(final int code, final String message) {
    return new Status(true, code, message);
  }

  public static Status success(final int code) {
    return new Status(true, code);
  }

  public static Status failure(final int code, final String message) {
    return new Status(false, code, message);
  }

  public static Status failure(final int code) {
    return new Status(false, code);
  }
}
