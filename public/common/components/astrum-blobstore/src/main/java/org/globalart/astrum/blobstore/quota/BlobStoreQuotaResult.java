
package org.globalart.astrum.blobstore.quota;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Holds result for the evaluation of {@link BlobStoreQuota}.
 *
 * @since 3.14
 */
public class BlobStoreQuotaResult
{
  private final boolean isViolation;

  private final String humanReadableMessage;

  private final String blobStoreName;

  public BlobStoreQuotaResult(
      final boolean isViolation,
      final String blobStoreName,
      final String humanReadableMessage)
  {
    this.isViolation = isViolation;
    this.blobStoreName = checkNotNull(blobStoreName);
    this.humanReadableMessage = checkNotNull(humanReadableMessage);
  }

  public boolean isViolation() {
    return this.isViolation;
  }

  public String getMessage() {
    return this.humanReadableMessage;
  }

  public String getBlobStoreName() {
    return this.blobStoreName;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "isViolation='" + isViolation + '\'' +
        ", message='" + humanReadableMessage + '\'' +
        ", blobStoreName='" + blobStoreName + '\'' +
        '}';
  }
}
