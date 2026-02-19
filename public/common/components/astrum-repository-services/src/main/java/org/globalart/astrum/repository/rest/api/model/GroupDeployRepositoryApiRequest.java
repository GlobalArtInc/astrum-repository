
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.28
 */
@JsonIgnoreProperties({"type"})
public class GroupDeployRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  public GroupDeployRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group)
  {
    super(name, format, online, storage, group);
  }

  @Override
  public GroupDeployAttributes getGroup() {
    return (GroupDeployAttributes) super.getGroup();
  }
}
