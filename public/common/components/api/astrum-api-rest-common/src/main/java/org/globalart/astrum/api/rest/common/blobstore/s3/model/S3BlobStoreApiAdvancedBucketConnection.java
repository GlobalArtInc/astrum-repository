
package org.globalart.astrum.api.rest.common.blobstore.s3.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static com.fasterxml.jackson.annotation.JsonInclude.Include.NON_NULL;

/**
 * Encapsulates S3 endpoint url, signer type and whether path-style access should be enabled for the specified S3
 * endpoint url.
 *
 * @since 3.20
 */
@JsonInclude(NON_NULL)
public class S3BlobStoreApiAdvancedBucketConnection
{
  @Schema(description = "A custom endpoint URL for third party object stores using the S3 API.")
  private final String endpoint;

  @Schema(
      description = "An API signature version which may be required for third party object stores using the S3 API.")
  private final String signerType;

  @Schema(description = "Setting this flag will result in path-style access being used for all requests.")
  private final Boolean forcePathStyle;

  @Schema(
      description = "Setting this value will override the default connection pool size of Astrum of the s3 client for this blobstore.")
  private final Integer maxConnectionPoolSize;

  @JsonCreator
  public S3BlobStoreApiAdvancedBucketConnection(
      @JsonProperty("endpoint") final String endpoint,
      @JsonProperty("signerType") final String signerType,
      @JsonProperty("forcePathStyle") final Boolean forcePathStyle,
      @JsonProperty("maxConnectionPoolSize") final Integer maxConnectionPoolSize)
  {
    this.endpoint = endpoint;
    this.signerType = signerType;
    this.forcePathStyle = forcePathStyle;
    this.maxConnectionPoolSize = maxConnectionPoolSize;
  }

  public String getEndpoint() {
    return endpoint;
  }

  public String getSignerType() {
    return signerType;
  }

  public Boolean getForcePathStyle() {
    return forcePathStyle;
  }

  public Integer getMaxConnectionPoolSize() {
    return maxConnectionPoolSize;
  }
}
