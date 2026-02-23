
package org.globalart.astrum.repository.search.internal;

import java.util.Set;

import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.query.SearchResultComponentGeneratorSupport;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

/**
 * @since 3.14
 */
@Primary
@Component
@Qualifier(DefaultSearchResultComponentGenerator.DEFAULT_SEARCH_RESULT_COMPONENT_GENERATOR_KEY)
public class DefaultSearchResultComponentGenerator
    extends SearchResultComponentGeneratorSupport
{
  public static final String DEFAULT_SEARCH_RESULT_COMPONENT_GENERATOR_KEY = "default";

  @Autowired
  public DefaultSearchResultComponentGenerator(
      final VariableResolverAdapterManager variableResolverAdapterManager,
      final RepositoryManager repositoryManager,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(variableResolverAdapterManager, repositoryManager, contentPermissionChecker);
  }

  @Override
  public ComponentSearchResult from(final ComponentSearchResult hit, final Set<String> componentIdSet) {
    hit.setRepositoryName(getPrivilegedRepositoryName(hit));
    return hit;
  }
}
