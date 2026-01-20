
package org.globalart.astrum.api.rest.common.blobstore.s3.model;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static com.fasterxml.jackson.annotation.JsonInclude.Include.NON_NULL;
import static org.globalart.astrum.blobstore.s3.internal.AmazonS3Factory.DEFAULT;

/**
 * Encapsulates the S3 bucket details for an s3 blob store.
 *
 * @since 3.20
 */
@JsonInclude(NON_NULL)
public class S3BlobStoreApiBucket
{
  @Valid
  @NotNull
  @Schema(description = "The AWS region to create a new S3 bucket in or an existing S3 bucket's region",
      example = DEFAULT, requiredMode = Schema.RequiredMode.REQUIRED)
  private final String region;

  @Valid
  @NotNull
  @Schema(description = "The name of the S3 bucket", requiredMode = Schema.RequiredMode.REQUIRED)
  private final String name;

  @Schema(description = "The S3 blob store (i.e S3 object) key prefix")
  private final String prefix;

  @JsonCreator
  public S3BlobStoreApiBucket(
      @JsonProperty("region") final String region,
      @JsonProperty("name") final String name,
      @JsonProperty("prefix") final String prefix)
  {
    this.region = region;
    this.name = name;
    this.prefix = prefix;
  }

  public String getRegion() {
    return region;
  }

  public String getName() {
    return name;
  }

  public String getPrefix() {
    return prefix;
  }
}
