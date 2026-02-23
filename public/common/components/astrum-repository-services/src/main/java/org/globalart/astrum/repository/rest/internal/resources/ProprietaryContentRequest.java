

package org.globalart.astrum.repository.rest.internal.resources;

import java.util.List;

import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.30
 */
public class ProprietaryContentRequest
{
  @NotNull
  private final List<String> proprietary;

  @NotNull
  private final List<String> nonProprietary;

  @JsonCreator
  public ProprietaryContentRequest(
      @JsonProperty("proprietary") final List<String> proprietary,
      @JsonProperty("nonProprietary") final List<String> nonProprietary)
  {
    this.proprietary = proprietary;
    this.nonProprietary = nonProprietary;
  }

  public List<String> getProprietary() {
    return proprietary;
  }

  public List<String> getNonProprietary() {
    return nonProprietary;
  }
}
