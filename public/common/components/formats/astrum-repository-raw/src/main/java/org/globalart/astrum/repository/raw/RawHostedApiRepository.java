
package org.globalart.astrum.repository.raw;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.constraints.NotNull;

/**
 * @since 3.41
 */
public class RawHostedApiRepository
    extends SimpleApiHostedRepository
{
  @NotNull
  private final RawAttributes raw;

  @JsonCreator
  public RawHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("raw") final RawAttributes raw)
  {
    super(name, RawFormat.NAME, url, online, storage, cleanup, component);
    this.raw = raw;
  }

  public RawAttributes getRaw() {
    return raw;
  }
}
