
package org.globalart.astrum.repository.apt;

import java.util.Collections;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;

/**
 * Common base for an Apt upload handlers
 *
 * @since 3.31
 */
public abstract class AptUploadHandlerSupport
    extends UploadHandlerSupport
{
  private UploadDefinition definition;

  private final VariableResolverAdapter variableResolverAdapter;

  private final ContentPermissionChecker contentPermissionChecker;

  public AptUploadHandlerSupport(
      final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.variableResolverAdapter = variableResolverAdapter;
    this.contentPermissionChecker = contentPermissionChecker;
  }

  protected void doValidation(final Repository repository, final String assetPath) {
    ensurePermitted(repository.getName(), AptFormat.NAME, assetPath, Collections.emptyMap());
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(AptFormat.NAME, false);
    }
    return definition;
  }

  public VariableResolverAdapter getVariableResolverAdapter() {
    return variableResolverAdapter;
  }

  public ContentPermissionChecker contentPermissionChecker() {
    return contentPermissionChecker;
  }
}
