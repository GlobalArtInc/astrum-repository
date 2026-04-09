
package org.globalart.astrum.repository.helm.internal.content.recipe;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.InputStream;
import java.util.stream.StreamSupport;


import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;
import org.globalart.astrum.repository.helm.internal.util.HelmAttributeParser;
import org.globalart.astrum.repository.helm.internal.util.HelmPathUtils;

import org.apache.commons.lang3.StringUtils;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Objects.requireNonNull;
import static org.globalart.astrum.repository.http.HttpResponses.created;
import static org.globalart.astrum.repository.http.HttpResponses.notFound;
import static org.globalart.astrum.repository.http.HttpResponses.ok;

/**
 * Helm Hosted Handlers
 *
 * @since 3.28
 */
@Component
public class HostedHandlers
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private HelmPathUtils helmPathUtils;

  private HelmAttributeParser helmPackageParser;

  @Autowired
  public HostedHandlers(final HelmPathUtils helmPathUtils, final HelmAttributeParser helmPackageParser) {
    this.helmPathUtils = checkNotNull(helmPathUtils);
    this.helmPackageParser = checkNotNull(helmPackageParser);
  }

  public final Handler get = context -> {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    String path;
    if (assetKind == AssetKind.HELM_INDEX) {
      path = "/index.yaml";
    }
    else {
      State state = context.getAttributes().require(State.class);
      path = helmPathUtils.contentFilePath(state, true);
    }
    Content content = context.getRepository().facet(HelmHostedFacet.class).get(path);

    return (content != null) ? ok(content) : notFound();
  };

  public final Handler upload = context -> {
    State state = context.getAttributes().require(State.class);
    String path = helmPathUtils.buildContentAssetPath(state);
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    context.getRepository().facet(HelmHostedFacet.class).upload(path, context.getRequest().getPayload(), assetKind);
    return ok();
  };

  public final Handler push = context -> {
    HelmHostedFacet hostedFacet = context.getRepository().facet(HelmHostedFacet.class);
    HelmContentFacet helmContentFacet = context.getRepository().facet(HelmContentFacet.class);

    PartPayload payload = getPartPayload(context);
    String fileName = payload.getName() != null ? payload.getName() : StringUtils.EMPTY;
    AssetKind assetKind = AssetKind.getAssetKindByFileName(fileName);
    try (TempBlob tempBlob = helmContentFacet.getTempBlob(payload);
         InputStream inputStream = tempBlob.get()) {
      HelmAttributes attributes = helmPackageParser.getAttributes(assetKind, inputStream);
      String path = hostedFacet.getPath(attributes, assetKind);

      hostedFacet.upload(path, tempBlob, attributes, payload, assetKind);
      return created();
    }
  };

  private PartPayload getPartPayload(final Context context) {
    if (!context.getRequest().isMultipart()) {
      throw new ValidationErrorsException("request should be multipart");
    }
    return StreamSupport.stream(requireNonNull(context.getRequest().getMultiparts()).spliterator(), false)
        .filter(partPayload -> "chart".equals(partPayload.getFieldName()))
        .findFirst().orElseThrow(() -> new ValidationErrorsException("chart field required"));
  }

  public final Handler delete = context -> {
    State state = context.getAttributes().require(State.class);
    String path = helmPathUtils.buildContentAssetPath(state);

    boolean deleted = context.getRepository().facet(HelmHostedFacet.class).delete(path);

    return (deleted) ? ok() : notFound();
  };
}
