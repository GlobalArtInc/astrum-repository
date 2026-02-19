
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.types.GroupType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * API Group Repository for simple formats which do not have custom attributes for groups.
 *
 * @since 3.20
 */
public class SimpleApiGroupRepository
    extends AbstractApiRepository
{
  @Schema
  @NotNull
  protected final StorageAttributes storage;

  @Schema
  @NotNull
  protected final GroupAttributes group;

  @JsonCreator
  public SimpleApiGroupRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group)
  {
    super(name, format, GroupType.NAME, url, online);
    this.storage = storage;
    this.group = group;
  }

  public StorageAttributes getStorage() {
    return storage;
  }

  public GroupAttributes getGroup() {
    return group;
  }
}
