
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.types.HostedType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.20
 */
@JsonIgnoreProperties({"type"})
public class HostedRepositoryApiRequest
    extends AbstractRepositoryApiRequest
{
  @NotNull
  @Valid
  private final HostedStorageAttributes storage;

  @Valid
  private final CleanupPolicyAttributes cleanup;

  @Valid
  private final ComponentAttributes component;

  @JsonCreator
  public HostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes)
  {
    super(name, format, HostedType.NAME, online);
    this.storage = storage;
    this.cleanup = cleanup;
    this.component = componentAttributes;
  }

  public HostedStorageAttributes getStorage() {
    return storage;
  }

  public CleanupPolicyAttributes getCleanup() {
    return cleanup;
  }

  public ComponentAttributes getComponent() {
    return component;
  }
}
