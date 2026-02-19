
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * API Group Deploy Repository for simple formats which do not have custom attributes for groups.
 *
 * @since 3.28
 */
public class SimpleApiGroupDeployRepository
    extends SimpleApiGroupRepository
{
  public SimpleApiGroupDeployRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group)
  {
    super(name, format, url, online, storage, group);
  }

  @Override
  public GroupDeployAttributes getGroup() {
    return (GroupDeployAttributes) super.getGroup();
  }
}
