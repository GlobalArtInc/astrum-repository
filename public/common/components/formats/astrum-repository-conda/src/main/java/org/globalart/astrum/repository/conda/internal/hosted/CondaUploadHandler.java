package org.globalart.astrum.repository.conda.internal.hosted;

import java.io.IOException;
import java.util.Collections;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.metadata.InvalidCondaPackageException;
import org.globalart.astrum.repository.conda.util.CondaPathUtils;
import org.globalart.astrum.repository.conda.util.CondaPathUtils.PackagePath;
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

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(CondaFormat.NAME)
public class CondaUploadHandler
    extends UploadHandlerSupport
{
  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public CondaUploadHandler(
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
    CondaHostedFacet hostedFacet = repository.facet(CondaHostedFacet.class);
    CondaContentFacet contentFacet = repository.facet(CondaContentFacet.class);

    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    String fileName = payload.getName() != null ? CondaPathUtils.lastSegment(payload.getName()) : "";
    AssetKind assetKind;
    try {
      assetKind = CondaPathUtils.packageKind(fileName);
    }
    catch (IllegalArgumentException e) {
      throw new ValidationErrorsException(e.getMessage());
    }

    try (TempBlob tempBlob = contentFacet.getTempBlob(payload)) {
      Map<String, Object> index;
      PackagePath packagePath;
      try {
        index = hostedFacet.readIndex(tempBlob, assetKind);
        packagePath = hostedFacet.packagePath(index, assetKind);
      }
      catch (InvalidCondaPackageException e) {
        throw new ValidationErrorsException(e.getMessage());
      }
      String path = packagePath.getPath();
      ensurePermitted(repository.getName(), CondaFormat.NAME, path,
          Map.of("name", packagePath.getName(), "version", packagePath.getVersion()));
      Content content = hostedFacet.store(packagePath, tempBlob, index);
      return new UploadResponse(Collections.singletonList(content), Collections.singletonList(path));
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(CondaFormat.NAME, false);
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
