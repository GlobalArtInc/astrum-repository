
package org.globalart.astrum.repository.maven.internal.search;

import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.function.Predicate;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.query.SearchResultComponentGeneratorSupport;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;

import static java.util.Objects.nonNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.14
 */
@Component
@Qualifier(Maven2Format.NAME)
public class Maven2SearchResultComponentGenerator
    extends SearchResultComponentGeneratorSupport
{
  @Autowired
  public Maven2SearchResultComponentGenerator(
      final VariableResolverAdapterManager variableResolverAdapterManager,
      final RepositoryManager repositoryManager,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(variableResolverAdapterManager, repositoryManager, contentPermissionChecker);
  }

  @Override
  public ComponentSearchResult from(final ComponentSearchResult hit, final Set<String> componentIdSet) {
    hit.setRepositoryName(getPrivilegedRepositoryName(hit));

    Optional<String> baseVersion = Optional.ofNullable(hit.getAnnotation("baseVersion"))
        .map(Object::toString)
        .filter(Objects::nonNull)
        .filter(((Predicate<String>) String::isEmpty).negate());

    if (baseVersion.isPresent()) {
      return createComponentForBaseVersion(hit, componentIdSet, baseVersion.get());
    }
    return hit;
  }

  private ComponentSearchResult createComponentForBaseVersion(
      final ComponentSearchResult hit,
      final Set<String> componentIdSet,
      final String baseVersion)
  {
    ComponentSearchResult component = null;
    String baseVersionId = hit.getRepositoryName() + ":" + hit.getGroup() + ":" + hit.getName() + ":" + baseVersion;

    if (!componentIdSet.contains(baseVersionId)) {
      boolean isSnapshot = isSnapshotId(baseVersionId);
      component = new ComponentSearchResult();

      component.setId(isSnapshot ? baseVersionId : hit.getId());
      component.setRepositoryName(hit.getRepositoryName());
      component.setGroup(hit.getGroup());
      component.setName(hit.getName());
      component.setFormat(hit.getFormat());
      component.setAssets(hit.getAssets());
      component.setLastModified(hit.getLastModified());

      component.setVersion(baseVersion);
    }

    return component;
  }

  public static boolean isSnapshotId(final String id) {
    return nonNull(id) && id.endsWith("-SNAPSHOT");
  }
}
