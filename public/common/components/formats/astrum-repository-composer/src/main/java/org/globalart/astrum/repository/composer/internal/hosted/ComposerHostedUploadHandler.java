
package org.globalart.astrum.repository.composer.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.Arrays;
import java.util.Collections;
import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.google.common.collect.ImmutableMap;
import com.google.common.io.ByteStreams;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.Collections.singletonList;
import static org.apache.commons.lang3.StringUtils.removeStart;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildZipballPath;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.getProjectToken;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.getVendorToken;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.getVersionToken;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.PACKAGE_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.PROJECT_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_REFERENCE_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_TYPE_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_URL_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VENDOR_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VERSION_TOKEN;

@Component
@Qualifier(ComposerFormat.NAME)
public class ComposerHostedUploadHandler
    extends UploadHandlerSupport
    implements Handler
{
  private final VariableResolverAdapter variableResolverAdapter;

  private final ContentPermissionChecker contentPermissionChecker;

  private UploadDefinition definition;

  @Autowired
  public ComposerHostedUploadHandler(
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(uploadDefinitionExtensions);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String vendor = getVendorToken(context);
    String project = getProjectToken(context);
    String version = getVersionToken(context);
    String sourceType = null;
    String sourceUrl = null;
    String sourceRef = null;
    Payload payload = null;

    Request request = checkNotNull(context.getRequest());
    if (request.isMultipart() && request.getMultiparts() != null) {
      for (PartPayload part : request.getMultiparts()) {
        if (SOURCE_TYPE_FIELD_NAME.equals(part.getFieldName())) {
          sourceType = readString(part);
        }
        else if (SOURCE_URL_FIELD_NAME.equals(part.getFieldName())) {
          sourceUrl = readString(part);
        }
        else if (SOURCE_REFERENCE_FIELD_NAME.equals(part.getFieldName())) {
          sourceRef = readString(part);
        }
        else if (PACKAGE_FIELD_NAME.equals(part.getFieldName())) {
          payload = readBytes(part);
        }
      }
    }
    else {
      payload = request.getPayload();
    }
    if (payload == null) {
      return HttpResponses.badRequest("Missing package archive");
    }

    upload(context.getRepository(), vendor, project, version, sourceType, sourceUrl, sourceRef, payload);
    return HttpResponses.ok();
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    AssetUpload assetUpload = upload.getAssetUploads().get(0);
    String vendor = checkNotNull(assetUpload.getField(VENDOR_TOKEN));
    String project = checkNotNull(assetUpload.getField(PROJECT_TOKEN));
    String version = checkNotNull(assetUpload.getField(VERSION_TOKEN));

    ensurePermitted(repository.getName(), ComposerFormat.NAME,
        removeStart(buildZipballPath(vendor, project, version), "/"),
        ImmutableMap.of(VENDOR_TOKEN, vendor, PROJECT_TOKEN, project, VERSION_TOKEN, version));

    FluentAsset asset =
        upload(repository, vendor, project, version, null, null, null, checkNotNull(assetUpload.getPayload()));
    return new UploadResponse(singletonList(asset.path()));
  }

  private FluentAsset upload(
      final Repository repository,
      final String vendor,
      final String project,
      final String version,
      final String sourceType,
      final String sourceUrl,
      final String sourceRef,
      final Payload payload) throws IOException
  {
    ComposerHostedFacet hostedFacet = repository.facet(ComposerHostedFacet.class);
    FluentAsset asset = hostedFacet.upload(vendor, project, version, sourceType, sourceUrl, sourceRef, payload);
    hostedFacet.rebuildProviderJson(vendor, project);
    hostedFacet.rebuildPackageJson(vendor, project);
    return asset;
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(
          ComposerFormat.NAME,
          false,
          Collections.emptyList(),
          Arrays.asList(
              new UploadFieldDefinition(VENDOR_TOKEN, false, UploadFieldDefinition.Type.STRING),
              new UploadFieldDefinition(PROJECT_TOKEN, false, UploadFieldDefinition.Type.STRING),
              new UploadFieldDefinition(VERSION_TOKEN, false, UploadFieldDefinition.Type.STRING)),
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

  private static BytesPayload readBytes(final PartPayload part) throws IOException {
    try (InputStream in = part.openInputStream()) {
      return new BytesPayload(ByteStreams.toByteArray(in), part.getContentType());
    }
  }

  private static String readString(final PartPayload part) throws IOException {
    try (InputStream in = part.openInputStream()) {
      return new String(ByteStreams.toByteArray(in), UTF_8).trim();
    }
  }
}
