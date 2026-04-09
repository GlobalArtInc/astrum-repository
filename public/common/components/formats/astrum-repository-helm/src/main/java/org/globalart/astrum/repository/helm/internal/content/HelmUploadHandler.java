
package org.globalart.astrum.repository.helm.internal.content;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.Set;


import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.HelmUploadHandlerSupport;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.content.recipe.HelmHostedFacet;
import org.globalart.astrum.repository.helm.internal.util.HelmAttributeParser;

import org.apache.commons.lang3.StringUtils;

import static org.globalart.astrum.repository.helm.internal.HelmFormat.NAME;
import static org.globalart.astrum.repository.helm.internal.util.HelmAttributeParser.validateAttributes;

/**
 * Support helm upload for web page
 /**
 * @since 3.28
 */
@Component
@Qualifier(NAME)
public class HelmUploadHandler
    extends HelmUploadHandlerSupport
{
  @Autowired
  public HelmUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      final HelmAttributeParser helmPackageParser,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(contentPermissionChecker, helmPackageParser, variableResolverAdapter, uploadDefinitionExtensions);
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    HelmHostedFacet facet = repository.facet(HelmHostedFacet.class);
    HelmContentFacet helmContentFacet = repository.facet(HelmContentFacet.class);

    PartPayload payload = upload.getAssetUploads().get(0).getPayload();

    String fileName = payload.getName() != null ? payload.getName() : StringUtils.EMPTY;
    AssetKind assetKind = AssetKind.getAssetKindByFileName(fileName);

    try (TempBlob tempBlob = helmContentFacet.getTempBlob(payload);
         InputStream inputStream = tempBlob.get()) {
      HelmAttributes attributes = validateAttributes(helmPackageParser.getAttributes(assetKind, inputStream));
      String path = facet.getPath(attributes, assetKind);

      ensurePermitted(repository.getName(), NAME, path, Collections.emptyMap());
      Content content = facet.upload(path, tempBlob, attributes, payload, assetKind);
      return new UploadResponse(Collections.singletonList(content), Collections.singletonList(path));
    }
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(NAME, false);
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
