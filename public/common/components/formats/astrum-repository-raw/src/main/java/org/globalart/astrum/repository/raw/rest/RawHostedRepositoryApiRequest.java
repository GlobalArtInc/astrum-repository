
package org.globalart.astrum.repository.raw.rest;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.24
 */
@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class RawHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  private final RawAttributes raw;

  @JsonCreator
  public RawHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("raw") final RawAttributes raw,
      @JsonProperty("component") final ComponentAttributes componentAttributes)
  {
    super(name, RawFormat.NAME, online, storage, cleanup, componentAttributes);
    this.raw = raw;
  }

  public RawAttributes getRaw() {
    return raw;
  }
}
