
package org.globalart.astrum.repository.search.query;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.AssetSearchResult;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.selector.VariableSource;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.15
 */
public abstract class SearchResultComponentGeneratorSupport
    implements SearchResultComponentGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  final VariableResolverAdapterManager variableResolverAdapterManager;

  final RepositoryManager repositoryManager;

  final ContentPermissionChecker contentPermissionChecker;

  public SearchResultComponentGeneratorSupport(
      final VariableResolverAdapterManager variableResolverAdapterManager,
      final RepositoryManager repositoryManager,
      final ContentPermissionChecker contentPermissionChecker)
  {
    this.variableResolverAdapterManager = checkNotNull(variableResolverAdapterManager);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  protected String getPrivilegedRepositoryName(final ComponentSearchResult source) {
    String repositoryName = source.getRepositoryName();
    String repositoryFormat = source.getFormat();

    List<AssetSearchResult> assets = source.getAssets();

    if (assets != null && !assets.isEmpty()) {
      VariableResolverAdapter variableResolverAdapter = variableResolverAdapterManager.get(repositoryFormat);

      VariableSource variableSource = variableResolverAdapter.fromSearchResult(source, assets.get(0));
      List<String> repositoryNames = new ArrayList<>(repositoryManager.findContainingGroups(repositoryName));
      repositoryNames.addFirst(repositoryName);
      for (String name : repositoryNames) {
        if (contentPermissionChecker.isPermitted(name, repositoryFormat, BreadActions.BROWSE, variableSource)) {
          return name;
        }
      }
    }

    return repositoryName;
  }
}
