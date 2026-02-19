
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.types.HostedType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * API Hosted Repository for simple formats which do not have custom attributes for hosted repositories.
 *
 * @since 3.20
 */
public class SimpleApiHostedRepository
    extends AbstractApiRepository
{
  @NotNull
  protected final HostedStorageAttributes storage;

  protected final CleanupPolicyAttributes cleanup;

  protected final ComponentAttributes component;

  @JsonCreator
  public SimpleApiHostedRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component)
  {
    super(name, format, HostedType.NAME, url, online);
    this.storage = storage;
    this.cleanup = cleanup;
    this.component = component;
  }

  public CleanupPolicyAttributes getCleanup() {
    return cleanup;
  }

  public HostedStorageAttributes getStorage() {
    return storage;
  }

  public ComponentAttributes getComponent() {
    return component;
  }
}
