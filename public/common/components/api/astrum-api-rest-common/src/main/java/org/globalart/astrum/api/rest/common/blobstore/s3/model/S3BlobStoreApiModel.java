
package org.globalart.astrum.api.rest.common.blobstore.s3.model;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreApiSoftQuota;
import org.globalart.astrum.validation.constraint.NamePatternConstants;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.blobstore.s3.internal.S3BlobStore.TYPE;

/**
 * Encapsulates the API payload for creating, reading and updating an S3 blob store.
 *
 * @since 3.20
 */
public class S3BlobStoreApiModel
{
  @NotNull
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @Schema(description = "The name of the S3 blob store.", example = "s3", requiredMode = Schema.RequiredMode.REQUIRED)
  private String name;

  @Schema(description = "Settings to control the soft quota.")
  private final BlobStoreApiSoftQuota softQuota;

  @Valid
  @NotNull
  @Schema(description = "The S3 specific configuration details for the S3 object that'll contain the blob store.")
  private final S3BlobStoreApiBucketConfiguration bucketConfiguration;

  @JsonCreator
  public S3BlobStoreApiModel(
      @JsonProperty("name") final String name,
      @JsonProperty("softQuota") final BlobStoreApiSoftQuota softQuota,
      @JsonProperty("bucketConfiguration") final S3BlobStoreApiBucketConfiguration bucketConfiguration)
  {
    this.name = name;
    this.softQuota = softQuota;
    this.bucketConfiguration = bucketConfiguration;
  }

  public String getName() {
    return name;
  }

  public BlobStoreApiSoftQuota getSoftQuota() {
    return softQuota;
  }

  public S3BlobStoreApiBucketConfiguration getBucketConfiguration() {
    return bucketConfiguration;
  }

  @Schema(description = "The blob store type.", example = TYPE)
  public String getType() {
    return TYPE;
  }

}
