
package org.globalart.astrum.repository.raw;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotNull;

public class RawGroupApiRepository
    extends SimpleApiGroupRepository
{
  @Schema
  @NotNull
  private final RawAttributes raw;

  @JsonCreator
  public RawGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("raw") final RawAttributes raw)
  {
    super(name, RawFormat.NAME, url, online, storage, group);
    this.raw = raw;
  }

  public RawAttributes getRaw() {
    return raw;
  }
}
