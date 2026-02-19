
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class SwiftAttributes
{
  public static final String REQUIRE_AUTHENTICATION = "requireAuthentication";

  @Schema(description = "Indicates if this repository requires authentication overriding anonymous access.",
      example = "false")
  private Boolean requireAuthentication;

  public SwiftAttributes() {
    this.requireAuthentication = null;
  }

  @JsonCreator
  public SwiftAttributes(
      @JsonProperty(REQUIRE_AUTHENTICATION) final Boolean requireAuthentication)
  {
    this.requireAuthentication = requireAuthentication;
  }

  public Boolean getRequireAuthentication() {
    return requireAuthentication;
  }

  public void setRequireAuthentication(final Boolean requireAuthentication) {
    this.requireAuthentication = requireAuthentication;
  }
}
