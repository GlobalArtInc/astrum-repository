package org.globalart.astrum.repository.rubygems.internal.hosted;

import java.io.IOException;
import java.util.Collections;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
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
@Qualifier(RubygemsFormat.NAME)
public class RubygemsUploadHandler
    extends UploadHandlerSupport
{
  private static final String ASSET_FIELD = "asset";

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public RubygemsUploadHandler(
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
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    FluentAsset asset;
    try {
      asset = repository.facet(RubygemsHostedFacet.class).push(payload,
          path -> ensurePermitted(repository.getName(), RubygemsFormat.NAME, path, Collections.emptyMap()));
    }
    catch (InvalidContentException e) {
      throw new ValidationErrorsException(ASSET_FIELD, e.getMessage());
    }
    Content content = asset.download();
    return new UploadResponse(List.of(content), List.of(asset.path()));
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(RubygemsFormat.NAME, false, List.of(), List.of(), null);
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
