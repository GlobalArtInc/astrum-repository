
package org.globalart.astrum.repository.selector;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;
import org.globalart.astrum.selector.VariableSource;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.stream;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import static org.globalart.astrum.security.BreadActions.READ;
import org.springframework.stereotype.Component;

/**
 * Provides various content related auth checks.
 *
 * @since 3.26
 */
@Component
public class ContentAuthHelper
{
  private final VariableResolverAdapterManager variableResolverAdapterManager;

  private final ContentPermissionChecker contentPermissionChecker;

  @Autowired
  public ContentAuthHelper(
      final VariableResolverAdapterManager variableResolverAdapterManager,
      final ContentPermissionChecker contentPermissionChecker)
  {
    this.variableResolverAdapterManager = checkNotNull(variableResolverAdapterManager);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  public boolean checkPathPermissions(final String path, final String format, final String... repositoryNames) {
    VariableResolverAdapter variableResolverAdapter = variableResolverAdapterManager.get(format);
    VariableSource variableSource = variableResolverAdapter.fromPath(path, format);
    return stream(repositoryNames).anyMatch(
        repositoryName -> contentPermissionChecker.isPermittedAnyOf(repositoryName, format, variableSource, BROWSE,
            READ));
  }

  public boolean checkPathPermissionsJexlOnly(final String path, final String format, final String... repositoryNames) {
    VariableResolverAdapter variableResolverAdapter = variableResolverAdapterManager.get(format);
    VariableSource variableSource = variableResolverAdapter.fromPath(path, format);
    return stream(repositoryNames).anyMatch(
        repositoryName -> contentPermissionChecker.isPermittedJexlOnlyAnyOf(repositoryName, format, variableSource,
            BROWSE, READ));
  }

}
