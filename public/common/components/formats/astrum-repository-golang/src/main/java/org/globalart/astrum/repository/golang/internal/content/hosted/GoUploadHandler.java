package org.globalart.astrum.repository.golang.internal.content.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.golang.internal.content.GoContentFacet;
import org.globalart.astrum.repository.golang.internal.metadata.GolangAttributes;
import org.globalart.astrum.repository.golang.internal.util.GoModuleZip;
import org.globalart.astrum.repository.golang.internal.util.GolangPathUtils;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition.Type;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.lang3.StringUtils.trimToNull;
import static org.globalart.astrum.repository.golang.internal.util.GolangPathUtils.assetPath;
import static org.globalart.astrum.repository.golang.internal.util.GolangPathUtils.escape;

@Component
@Qualifier(GolangFormat.NAME)
public class GoUploadHandler
    extends UploadHandlerSupport
{
  private static final String MODULE = "module";

  private static final String VERSION = "version";

  private static final String FIELD_GROUP_NAME = "Module";

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public GoUploadHandler(
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
    GoContentFacet contentFacet = repository.facet(GoContentFacet.class);
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();

    try (TempBlob zipBlob = contentFacet.ingest(payload)) {
      GoModuleZip moduleZip;
      try (InputStream in = zipBlob.get()) {
        moduleZip = GoModuleZip.read(in);
      }
      GolangAttributes attributes = new GolangAttributes();
      String module = trimToNull(upload.getFields().get(MODULE));
      String version = trimToNull(upload.getFields().get(VERSION));
      attributes.setModule(module == null ? moduleZip.module() : module);
      attributes.setVersion(version == null ? moduleZip.version() : version);
      GolangPathUtils.validate(attributes);

      String path = assetPath(escape(attributes.getModule()), escape(attributes.getVersion()), "zip");
      ensurePermitted(repository.getName(), GolangFormat.NAME, path, Collections.emptyMap());

      FluentAsset asset = contentFacet.upload(attributes, zipBlob);
      return new UploadResponse(Collections.singletonList(asset.download()), Collections.singletonList(path));
    }
    catch (InvalidContentException e) {
      throw new ValidationErrorsException(e.getMessage());
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(GolangFormat.NAME, false,
          List.of(
              new UploadFieldDefinition(MODULE, "Module path, read from the zip when empty", true, Type.STRING,
                  FIELD_GROUP_NAME),
              new UploadFieldDefinition(VERSION, "Module version, read from the zip when empty", true, Type.STRING,
                  FIELD_GROUP_NAME)),
          Collections.emptyList(),
          null);
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
