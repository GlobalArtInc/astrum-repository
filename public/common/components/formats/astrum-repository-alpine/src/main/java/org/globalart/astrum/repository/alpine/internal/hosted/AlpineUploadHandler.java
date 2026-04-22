package org.globalart.astrum.repository.alpine.internal.hosted;

import java.io.IOException;
import java.util.Collections;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.internal.AlpinePathUtils;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
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

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineUploadHandler
    extends UploadHandlerSupport
{
  public static final String DIRECTORY = "directory";

  private static final String DIRECTORY_HELP_TEXT =
      "Architecture directory of the package, for example v3.20/main/x86_64";

  private static final String FIELD_GROUP_NAME = "Component attributes";

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public AlpineUploadHandler(
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
    String directoryField = upload.getFields().get(DIRECTORY);
    if (directoryField == null || directoryField.isBlank()) {
      throw new ValidationErrorsException(DIRECTORY, "Directory is required");
    }
    String directory;
    try {
      directory = AlpinePathUtils.normalize(directoryField.trim());
    }
    catch (IllegalArgumentException e) {
      throw new ValidationErrorsException(DIRECTORY, e.getMessage());
    }
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    Content content = repository.facet(AlpineHostedFacet.class).put(directory, null, payload,
        path -> ensurePermitted(repository.getName(), AlpineFormat.NAME, path, Collections.emptyMap()));
    Asset asset = content.getAttributes().require(Asset.class);
    return new UploadResponse(List.of(content), List.of(asset.path()));
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(AlpineFormat.NAME, false,
          List.of(new UploadFieldDefinition(DIRECTORY, DIRECTORY_HELP_TEXT, false, Type.STRING, FIELD_GROUP_NAME)),
          List.of(), null);
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
