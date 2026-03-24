
package org.globalart.astrum.repository.maven.internal.datastore;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.director.ContentDirector;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.maven.MavenFacet;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import org.apache.commons.lang3.tuple.Pair;

import static org.globalart.astrum.repository.maven.VersionPolicy.MIXED;
import static org.globalart.astrum.repository.maven.VersionPolicy.RELEASE;
import static org.globalart.astrum.repository.maven.VersionPolicy.SNAPSHOT;
import static org.globalart.astrum.repository.maven.internal.datastore.MavenFacetUtils.isSnapshot;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven content director for datastore implementation.
 *
 * @since 3.31
 */
@org.springframework.stereotype.Component
@Qualifier(Maven2Format.NAME)
public class MavenContentDirector
    implements ContentDirector
{
  @Override
  public boolean allowMoveTo(final Repository destination) {
    return true;
  }

  @Override
  public boolean allowMoveTo(final FluentComponent component, final Repository destination) {
    VersionPolicy versionPolicy = destination.facet(MavenFacet.class).getVersionPolicy();
    if (MIXED.equals(versionPolicy)) {
      return true;
    }
    if (isSnapshot(component)) {
      return SNAPSHOT.equals(versionPolicy);
    }
    else {
      return RELEASE.equals(versionPolicy);
    }
  }

  @Override
  public boolean allowMoveFrom(final Repository source) {
    return true;
  }

  @Override
  public void afterMove(final List<Map<String, String>> components, final Repository destination) {
    MavenMetadataRebuildFacet facet = destination.facet(MavenMetadataRebuildFacet.class);
    components.stream()
        .map(component -> Pair.of(component.get("group"), component.get("name")))
        .distinct()
        .forEach(pair -> facet.rebuildMetadata(pair.getLeft(), pair.getRight(), null, false));
  }

  @Override
  public FluentComponent copyComponent(
      final Component source,
      final Repository destination)
  {
    return destination.facet(MavenContentFacet.class).copy(source);
  }
}
