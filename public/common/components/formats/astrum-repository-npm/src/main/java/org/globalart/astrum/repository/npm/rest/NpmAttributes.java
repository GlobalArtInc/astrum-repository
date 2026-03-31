package org.globalart.astrum.repository.npm.rest;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

@JsonIgnoreProperties(ignoreUnknown = true)
@JsonInclude(Include.NON_NULL)
public class NpmAttributes
{
  public static final String NPM = "npm";

  public static final String REMOVE_QUARANTINED = "removeQuarantined";

  public static final String REMOVE_NON_CATALOGED = "removeNonCataloged";

  @Schema(description = "Remove quarantined versions from the npm package metadata", example = "false")
  private final Boolean removeQuarantined;

  @Schema(description = "Remove non-cataloged versions from the npm package metadata", example = "false")
  private final Boolean removeNonCataloged;

  @JsonCreator
  public NpmAttributes(
      @JsonProperty(REMOVE_QUARANTINED) final Boolean removeQuarantined,
      @JsonProperty(REMOVE_NON_CATALOGED) final Boolean removeNonCataloged)
  {
    this.removeQuarantined = removeQuarantined;
    this.removeNonCataloged = removeNonCataloged;
  }

  public Boolean getRemoveQuarantined() {
    return removeQuarantined;
  }

  public Boolean getRemoveNonCataloged() {
    return removeNonCataloged;
  }
}
