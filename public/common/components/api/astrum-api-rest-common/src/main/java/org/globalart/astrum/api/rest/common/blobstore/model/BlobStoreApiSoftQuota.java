
package org.globalart.astrum.api.rest.common.blobstore.model;

import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotBlank;

import org.globalart.astrum.blobstore.quota.internal.SpaceRemainingQuota;
import org.globalart.astrum.blobstore.quota.internal.SpaceUsedQuota;

import io.swagger.v3.oas.annotations.media.Schema;

/**
 * @since 3.19
 */
public class BlobStoreApiSoftQuota
{
  @NotBlank
  @Schema(description = "The type to use such as " + SpaceRemainingQuota.ID + ", or " + SpaceUsedQuota.ID
      + SpaceUsedQuota.ID)
  private String type;

  @Min(0)
  @Schema(description = "The limit in MB.")
  private Long limit;

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public Long getLimit() {
    return limit;
  }

  public void setLimit(final Long limit) {
    this.limit = limit;
  }
}
