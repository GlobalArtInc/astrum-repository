
package org.globalart.astrum.content.maven.internal.search;

import java.util.Optional;
import java.util.regex.Pattern;
import java.util.stream.Stream;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponentBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.search.ComponentFinder;
import org.globalart.astrum.repository.content.search.DefaultComponentFinder;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.search.Maven2SearchResultComponentGenerator;
import org.globalart.astrum.repository.types.GroupType;

import static java.util.Comparator.reverseOrder;

import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven {@link ComponentFinder} that also includes all snapshots with the same base version.
 *
 * @since 3.26
 */
@Component
@Qualifier(Maven2Format.NAME)
public class Maven2ComponentFinder
    extends DefaultComponentFinder
{
  public static final Pattern SNAPSHOT_TIMESTAMP = Pattern.compile("^(.*-)?([0-9]{8}\\.[0-9]{6}-[0-9]+)$");

  @Override
  public Stream<FluentComponent> findComponentsByModel(
      final Repository repository,
      final String searchComponentId,
      final String namespace,
      final String name,
      final String version)
  {
    // we take a special approach if the search id is a snapshot coordinate
    if (Maven2SearchResultComponentGenerator.isSnapshotId(searchComponentId)) {
      FluentComponents components = contentFacet(repository).components();
      FluentComponentBuilder builder = components.name(name).namespace(namespace);

      String versionPrefix = version.replace("SNAPSHOT", "");

      if (GroupType.NAME.equals(repository.getType().getValue())) {
        return components.memberVersions(namespace, name)
            .stream()
            .filter(v -> v.startsWith(versionPrefix))
            .filter(v -> SNAPSHOT_TIMESTAMP.matcher(v).matches() || v.equals(version))
            .sorted(reverseOrder())
            .map(v -> builder.version(v).findInMembers())
            .filter(Optional::isPresent)
            .map(Optional::get);
      }

      // find timestamped versions that match the base version and fetch their components
      return components.versions(namespace, name)
          .stream()
          .filter(v -> v.startsWith(versionPrefix))
          .filter(v -> SNAPSHOT_TIMESTAMP.matcher(v).matches() || v.equals(version))
          .sorted(reverseOrder())
          .map(v -> builder.version(v).find())
          .filter(Optional::isPresent)
          .map(Optional::get);
    }

    return super.findComponentsByModel(repository, searchComponentId, namespace, name, version);
  }
}
