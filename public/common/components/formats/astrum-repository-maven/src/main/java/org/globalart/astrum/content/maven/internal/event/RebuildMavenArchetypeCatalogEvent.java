
package org.globalart.astrum.content.maven.internal.event;

/**
 * Event used to indicate when the archetype catalog for a given maven repository should be rebuilt.
 *
 * @since 3.26
 */
public final class RebuildMavenArchetypeCatalogEvent
{
  private final String repositoryName;

  public RebuildMavenArchetypeCatalogEvent(final String repositoryName) {
    this.repositoryName = repositoryName;
  }

  public String getRepositoryName() {
    return repositoryName;
  }
}
