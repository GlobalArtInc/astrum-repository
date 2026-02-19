
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model which describes replication capabilities for a repository.
 */
public class ReplicationAttributes
{
  @Schema(description = "Whether pre-emptive pull is enabled", example = "false")
  @NotNull
  protected final Boolean preemptivePullEnabled;

  @Schema(description = "Regular Expression of Asset Paths to pull pre-emptively pull")
  protected final String assetPathRegex;

  @JsonCreator
  public ReplicationAttributes(
      @JsonProperty("preemptivePullEnabled") final Boolean preemptivePull,
      @JsonProperty("assetPathRegex") final String assetPathRegex)
  {
    this.preemptivePullEnabled = preemptivePull;
    this.assetPathRegex = assetPathRegex;
  }

  public Boolean getPreemptivePullEnabled() {
    return preemptivePullEnabled;
  }

  public String getAssetPathRegex() {
    return assetPathRegex;
  }

}
