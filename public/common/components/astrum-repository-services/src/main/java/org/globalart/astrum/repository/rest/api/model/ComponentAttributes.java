
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model for attributes applying to components.
 *
 * @since 3.30
 */
public class ComponentAttributes
{
  @Schema(
      description = "Components in this repository count as proprietary for namespace conflict attacks (requires GlobalArt Astrum Firewall)")
  protected Boolean proprietaryComponents;

  @JsonCreator
  public ComponentAttributes(
      @JsonProperty(value = "proprietaryComponents" /*
                                                     * ASTRUM-46395 TODO: defaultValue moved to @Schema in OpenAPI
                                                     * 3.x: @Parameter(schema = @Schema(defaultValue = "false"))
                                                     */) final Boolean proprietaryComponents)
  {
    this.proprietaryComponents = proprietaryComponents;
  }

  public Boolean getProprietaryComponents() {
    return proprietaryComponents;
  }
}
