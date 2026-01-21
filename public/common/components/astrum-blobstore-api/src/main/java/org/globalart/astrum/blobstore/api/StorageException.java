
package org.globalart.astrum.blobstore.api;

import javax.annotation.Nullable;

/**
 * An exception used internally in blob stores for client failures, not intended for other usages.
 */
public class StorageException
    extends RuntimeException
{
  private final ErrorCode code;

  private final String path;

  private StorageException(final String path, final ErrorCode code) {
    this(path, code, null);
  }

  private StorageException(final String path, final ErrorCode code, @Nullable final Throwable cause) {
    super(code.message(path), cause);
    this.code = code;
    this.path = path;
  }

  public ErrorCode code() {
    return code;
  }

  public String path() {
    return path;
  }

  public static StorageException notFound(final String path) {
    return new StorageException(path, ErrorCode.NOT_FOUND);
  }

  public static StorageException timeout(final String path, @Nullable final Throwable e) {
    return new StorageException(path, ErrorCode.TIMEOUT, e);
  }

  public enum ErrorCode
  {
    NOT_FOUND("Missing path %s"),

    TIMEOUT("Timeout while retrieving %s");

    private final String message;

    ErrorCode(final String message) {
      this.message = message;
    }

    public String message(final String path) {
      return message.formatted(path);
    }
  }
}
