
package org.globalart.astrum.api.rest.common.blobstore.model;

import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;

import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.14
 */
public class BlobStoreQuotaResultXO
{
  private boolean isViolation;

  @NotEmpty
  private String message;

  @NotEmpty
  private String blobStoreName;

  public boolean getIsViolation() {
    return isViolation;
  }

  public void setIsViolation(final boolean isViolation) {
    this.isViolation = isViolation;
  }

  public String getMessage() {
    return message;
  }

  public void setMessage(final String message) {
    this.message = message;
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }

  public void setBlobStoreName(String blobStoreName) {
    this.blobStoreName = blobStoreName;
  }

  public static BlobStoreQuotaResultXO asQuotaXO(final BlobStoreQuotaResult result) {
    BlobStoreQuotaResultXO blobStoreQuotaResultXO = new BlobStoreQuotaResultXO();
    blobStoreQuotaResultXO.setIsViolation(result.isViolation());
    blobStoreQuotaResultXO.setMessage(result.getMessage());
    blobStoreQuotaResultXO.setBlobStoreName(result.getBlobStoreName());
    return blobStoreQuotaResultXO;
  }

  public static BlobStoreQuotaResultXO asNoQuotaXO(final String blobStoreName) {
    BlobStoreQuotaResultXO blobStoreQuotaResultXO = new BlobStoreQuotaResultXO();
    blobStoreQuotaResultXO.setIsViolation(false);
    blobStoreQuotaResultXO.setMessage("Blob store " + blobStoreName + " has no quota");
    blobStoreQuotaResultXO.setBlobStoreName(blobStoreName);
    return blobStoreQuotaResultXO;
  }
}
