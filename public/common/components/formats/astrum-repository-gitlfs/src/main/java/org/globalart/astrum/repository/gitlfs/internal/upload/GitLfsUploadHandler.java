package org.globalart.astrum.repository.gitlfs.internal.upload;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsContentFacet;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsObjects;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

/**
 * Uploads a file through the components API as the object its SHA-256 names, which is how objects can be stored
 * without a git client.
 */
@Component
@Qualifier(GitLfsFormat.NAME)
public class GitLfsUploadHandler
    extends UploadHandlerSupport
{
  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public GitLfsUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    GitLfsContentFacet content = repository.facet(GitLfsContentFacet.class);
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    try (TempBlob blob = content.ingest(payload)) {
      String path = GitLfsObjects.assetPath(blob.getHashes().get(SHA256).toString());
      ensurePermitted(repository.getName(), GitLfsFormat.NAME, path, Map.of());
      FluentAsset asset = content.put(blob);
      return new UploadResponse(List.of(asset.download()), List.of(path));
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(GitLfsFormat.NAME, false);
    }
    return definition;
  }

  @Override
  public VariableResolverAdapter getVariableResolverAdapter() {
    return variableResolverAdapter;
  }

  @Override
  public ContentPermissionChecker contentPermissionChecker() {
    return contentPermissionChecker;
  }
}
