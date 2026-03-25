
package org.globalart.astrum.repository.maven.rest;

import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Maven group repository API request.
 * Maven group repositories do not use versionPolicy, layoutPolicy, or contentDisposition fields.
 *
 * @since 3.24
 */
@JsonIgnoreProperties({"format", "type"})
public class MavenGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  @JsonCreator
  public MavenGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group)
  {
    super(name, Maven2Format.NAME, online, storage, group);
  }
}
