package org.globalart.astrum.repository.huggingface.internal;

import javax.annotation.Nullable;

/**
 * A request the Hub protocol answers with an error: the status, the {@code X-Error-Code} clients map to their
 * exceptions and, for missing files, the commit that was looked at.
 */
public class HuggingFaceException
    extends RuntimeException
{
  public static final String ENTRY_NOT_FOUND = "EntryNotFound";

  public static final String REVISION_NOT_FOUND = "RevisionNotFound";

  public static final String REPO_NOT_FOUND = "RepoNotFound";

  private static final String SEPARATOR = ": ";

  private final int status;

  @Nullable
  private final String errorCode;

  @Nullable
  private final String commit;

  public HuggingFaceException(final int status, @Nullable final String errorCode, final String message) {
    this(status, errorCode, message, null);
  }

  public HuggingFaceException(final int status, @Nullable final String errorCode, final String message,
                              @Nullable final String commit)
  {
    super(message);
    this.status = status;
    this.errorCode = errorCode;
    this.commit = commit;
  }

  public static HuggingFaceException entryNotFound(final String commit, final String path) {
    return new HuggingFaceException(404, ENTRY_NOT_FOUND, path + " does not exist at commit " + commit, commit);
  }

  public int getStatus() {
    return status;
  }

  @Nullable
  public String getErrorCode() {
    return errorCode;
  }

  @Nullable
  public String getCommit() {
    return commit;
  }

  /**
   * The error as one line, kept in the negative cache.
   */
  public String encode() {
    return errorCode == null ? getMessage() : errorCode + SEPARATOR + getMessage();
  }

  public static HuggingFaceException decode(final int status, @Nullable final String encoded) {
    String text = encoded == null ? "Not found" : encoded;
    int separator = text.indexOf(SEPARATOR);
    if (separator > 0 && text.substring(0, separator).matches("[A-Za-z]+")) {
      return new HuggingFaceException(status, text.substring(0, separator), text.substring(separator + 2));
    }
    return new HuggingFaceException(status, null, text);
  }
}
