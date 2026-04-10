
package org.globalart.astrum.repository.apt.api;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * REST API model of apt-specific hosted configuration.
 *
 * @since 3.20
 */
public class AptHostedRepositoriesAttributes
{
  @Schema(description = "Distribution name for this repository. " +
      "Defines the distribution path used in metadata structure (e.g., dists/{distribution}/Release). " +
      "Clients must configure this exact distribution name in their APT sources.",
      example = "bionic",
      required = true)
  @NotEmpty
  protected final String distribution;

  @JsonCreator
  public AptHostedRepositoriesAttributes(@JsonProperty("distribution") final String distribution) {
    this.distribution = distribution;
  }

  public String getDistribution() {
    return distribution;
  }
}
