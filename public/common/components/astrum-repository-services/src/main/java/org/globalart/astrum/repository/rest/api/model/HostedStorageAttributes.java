
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import org.globalart.astrum.repository.config.WritePolicy;

import java.util.Objects;

/**
 * REST API model for describing storage of hosted repositories.
 *
 * @since 3.20
 */
public class HostedStorageAttributes
    extends StorageAttributes
{
  @Schema(description = "Controls if deployments of and updates to assets are allowed",
      example = "ALLOW_ONCE")
  @NotNull
  protected final String writePolicy;

  @JsonCreator
  public HostedStorageAttributes(
      @JsonProperty("blobStoreName") final String blobStoreName,
      @JsonProperty("strictContentTypeValidation") final Boolean strictContentTypeValidation,
      @JsonProperty("writePolicy") final String writePolicy)
  {
    super(blobStoreName, strictContentTypeValidation);

    Objects.requireNonNull(writePolicy, "writePolicy is required");
    this.writePolicy = WritePolicy.fromString(writePolicy).name();
  }

  public String getWritePolicy() {
    return writePolicy;
  }
}
