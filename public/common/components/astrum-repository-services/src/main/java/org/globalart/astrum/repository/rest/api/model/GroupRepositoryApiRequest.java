
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.types.GroupType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * @since 3.20
 */
@JsonIgnoreProperties({"type"})
public class GroupRepositoryApiRequest
    extends AbstractRepositoryApiRequest
{
  @Schema
  @NotNull
  @Valid
  protected final StorageAttributes storage;

  @Schema
  @NotNull
  @Valid
  protected final GroupAttributes group;

  @JsonCreator
  public GroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group)
  {
    super(name, format, GroupType.NAME, online);
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
