
package org.globalart.astrum.repository.search.sql.store;

import java.time.OffsetDateTime;

/**
 * One distinct component version with the repositories containing it.
 */
public class ComponentVersionData
{
  private String version;

  private String normalisedVersion;

  private String repositories;

  private OffsetDateTime lastModified;

  public String getVersion() {
    return version;
  }

  public void setVersion(final String version) {
    this.version = version;
  }

  public String getNormalisedVersion() {
    return normalisedVersion;
  }

  public void setNormalisedVersion(final String normalisedVersion) {
    this.normalisedVersion = normalisedVersion;
  }

  /**
   * Comma-separated repository names, as aggregated by the database. Repository names cannot
   * contain a comma (see NamePatternConstants.REGEX), so this is unambiguous to split.
   */
  public String getRepositories() {
    return repositories;
  }

  public void setRepositories(final String repositories) {
    this.repositories = repositories;
  }

  public OffsetDateTime getLastModified() {
    return lastModified;
  }

  public void setLastModified(final OffsetDateTime lastModified) {
    this.lastModified = lastModified;
  }
}
