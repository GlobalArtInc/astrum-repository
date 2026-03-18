
package org.globalart.astrum.repository.raw.rest;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
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
public class RawGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  private final RawAttributes raw;

  @JsonCreator
  public RawGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("raw") final RawAttributes raw)
  {
    super(name, RawFormat.NAME, online, storage, group);
    this.raw = raw;
  }

  public RawAttributes getRaw() {
    return raw;
  }
}
