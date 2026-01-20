
package org.globalart.astrum.api.rest.common.blobstore.s3.model;

import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class S3BlobStoreApiFailoverBucket
{
  @NotNull
  @Schema(description = "The region containing the bucket")
  private final String region;

  @NotNull
  @Schema(description = "The name of the bucket in the region")
  private final String bucketName;

  public S3BlobStoreApiFailoverBucket(
      @JsonProperty("region") final String region,
      @JsonProperty("bucketName") final String bucketName)
  {
    this.region = region;
    this.bucketName = bucketName;
  }

  public String getRegion() {
    return region;
  }

  public String getBucketName() {
    return bucketName;
  }
}
