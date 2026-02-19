
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model for describing storage of docker hosted repositories.
 *
 * @since 3.41
 */
public class DockerHostedStorageAttributes
    extends HostedStorageAttributes
{
  @Schema(
      description = "Whether to allow redeploying the 'latest' tag but defer to the Deployment Policy for all other tags",
      example = "true")
  protected final Boolean latestPolicy;

  @JsonCreator
  public DockerHostedStorageAttributes(
      @JsonProperty("blobStoreName") final String blobStoreName,
      @JsonProperty("strictContentTypeValidation") final Boolean strictContentTypeValidation,
      @JsonProperty("writePolicy") final String writePolicy,
      @JsonProperty("latestPolicy") final Boolean latestPolicy)
  {
    super(blobStoreName, strictContentTypeValidation, writePolicy);
    this.latestPolicy = latestPolicy;
  }

  public Boolean getLatestPolicy() {
    return latestPolicy;
  }
}
