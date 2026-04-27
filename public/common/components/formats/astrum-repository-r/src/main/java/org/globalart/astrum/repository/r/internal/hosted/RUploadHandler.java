
package org.globalart.astrum.repository.r.internal.hosted;

import java.io.IOException;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition.Type;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.buildPath;

@Component
@Qualifier(RFormat.NAME)
public class RUploadHandler
    extends UploadHandlerSupport
{
  static final String PATH_ID = "pathId";

  static final String PACKAGE_PATH_DISPLAY = "Package Path";

  private static final String PACKAGE_PATH_HELP_TEXT =
      "Directory of the package, for example src/contrib or bin/windows/contrib/4.4";

  private final VariableResolverAdapter variableResolverAdapter;

  private final ContentPermissionChecker contentPermissionChecker;

  private UploadDefinition definition;

  @Autowired
  public RUploadHandler(
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    AssetUpload assetUpload = upload.getAssetUploads().get(0);
    PartPayload payload = assetUpload.getPayload();
    Map<String, String> fields = assetUpload.getFields();
    String uploadPath = fields.get(PATH_ID);
    if (uploadPath == null || uploadPath.isBlank()) {
      throw new ValidationErrorsException(PATH_ID, "Package Path is required");
    }
    if (payload.getName() == null || payload.getName().isBlank()) {
      throw new ValidationErrorsException("File name is required");
    }
    String assetPath = buildPath(uploadPath.trim(), payload.getName());
    try {
      Content content = repository.facet(RHostedFacet.class).upload(assetPath, payload,
          path -> ensurePermitted(repository.getName(), RFormat.NAME, path, Collections.emptyMap()));
      Asset asset = content.getAttributes().require(Asset.class);
      return new UploadResponse(List.of(content), List.of(asset.path()));
    }
    catch (InvalidContentException e) {
      throw new ValidationErrorsException(e.getMessage());
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      List<UploadFieldDefinition> assetFields = Collections.singletonList(
          new UploadFieldDefinition(PATH_ID, PACKAGE_PATH_DISPLAY, PACKAGE_PATH_HELP_TEXT, false, Type.STRING, null));
      definition = getDefinition(RFormat.NAME, false, Collections.emptyList(), assetFields, null);
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
