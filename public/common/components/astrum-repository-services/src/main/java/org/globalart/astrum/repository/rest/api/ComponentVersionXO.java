
package org.globalart.astrum.repository.rest.api;

import java.time.OffsetDateTime;
import java.util.List;

import io.swagger.v3.oas.annotations.media.Schema;

/**
 * One distinct version of a component and the repositories containing it.
 */
@Schema(description = "One distinct version of a component and the repositories containing it")
public class ComponentVersionXO
{
  @Schema(description = "The component version")
  private String version;

  @Schema(description = "The most recent modification time across the repositories holding this version")
  private OffsetDateTime lastUpdated;

  @Schema(description = "Names of the repositories containing this version")
  private List<String> repositories;

  public String getVersion() {
    return version;
  }

  public void setVersion(final String version) {
    this.version = version;
  }

  public OffsetDateTime getLastUpdated() {
    return lastUpdated;
  }

  public void setLastUpdated(final OffsetDateTime lastUpdated) {
    this.lastUpdated = lastUpdated;
  }

  public List<String> getRepositories() {
    return repositories;
  }

  public void setRepositories(final List<String> repositories) {
    this.repositories = repositories;
  }
}
