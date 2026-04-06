package org.globalart.astrum.repository.nuget.internal.upload;

import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.NugetPaths;
import org.globalart.astrum.repository.nuget.internal.content.NugetContentFacet;
import org.globalart.astrum.repository.nuget.internal.metadata.InvalidNugetPackageException;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata;
import org.globalart.astrum.repository.nuget.internal.metadata.NuspecParser;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetUploadHandler
    extends UploadHandlerSupport
{
  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public NugetUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.contentPermissionChecker = contentPermissionChecker;
    this.variableResolverAdapter = variableResolverAdapter;
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    NugetContentFacet content = repository.facet(NugetContentFacet.class);
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    try (TempBlob blob = content.ingest(payload)) {
      NugetPackageMetadata metadata;
      try (InputStream in = blob.get()) {
        metadata = NuspecParser.parsePackage(in);
      }
      catch (InvalidNugetPackageException e) {
        throw new ValidationErrorsException("Invalid NuGet package: " + e.getMessage());
      }
      String path = NugetPaths.packagePath(metadata.getId(), metadata.getVersion().toNormalizedString());
      ensurePermitted(repository.getName(), NugetFormat.NAME, path, Collections.emptyMap());
      Content stored = content.putPackage(blob, metadata, null);
      return new UploadResponse(Collections.singletonList(stored), Collections.singletonList(path));
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(NugetFormat.NAME, false);
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

  @Override
  public boolean supportsExportImport() {
    return true;
  }
}
