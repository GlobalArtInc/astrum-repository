
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * REST API model describing a repository's storage settings.
 *
 * @since 3.20
 */
public class StorageAttributes
{
  @Schema(description = "Blob store used to store repository contents", example = "default",
      requiredMode = Schema.RequiredMode.REQUIRED)
  @NotEmpty
  protected String blobStoreName;

  @Schema(description = "Whether to validate uploaded content's MIME type appropriate for the repository format",
      example = "true")
  @NotNull
  protected Boolean strictContentTypeValidation;

  @JsonCreator
  public StorageAttributes(
      @JsonProperty("blobStoreName") final String blobStoreName,
      @JsonProperty("strictContentTypeValidation") final Boolean strictContentTypeValidation)
  {
    this.blobStoreName = blobStoreName;
    this.strictContentTypeValidation = strictContentTypeValidation;
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }

  public Boolean getStrictContentTypeValidation() {
    return strictContentTypeValidation;
  }
}
