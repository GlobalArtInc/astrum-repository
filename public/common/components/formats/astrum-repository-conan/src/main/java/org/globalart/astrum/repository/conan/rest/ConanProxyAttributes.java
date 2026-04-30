package org.globalart.astrum.repository.conan.rest;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class ConanProxyAttributes
{
  @Schema(description = "Conan protocol version of the remote repository", example = "V2",
      allowableValues = {"V1", "V2"})
  private final String conanVersion;

  @JsonCreator
  public ConanProxyAttributes(@JsonProperty("conanVersion") final String conanVersion) {
    this.conanVersion = conanVersion;
  }

  public String getConanVersion() {
    return conanVersion;
  }
}
