
package org.globalart.astrum.repository.golang.internal.content.hosted;

import java.util.Optional;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.golang.AssetKind;
import org.globalart.astrum.repository.golang.internal.content.GoContentFacet;
import org.globalart.astrum.repository.golang.internal.util.GolangPathUtils;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static org.globalart.astrum.repository.golang.AssetKind.PACKAGE;
import static org.globalart.astrum.repository.http.HttpResponses.created;
import static org.globalart.astrum.repository.http.HttpResponses.notFound;

@Component
public class GoHostedHandlers
{
  public final Handler get = context -> {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    Optional<Content> content = switch (assetKind) {
      case INFO -> getInfo(context);
      case MODULE, PACKAGE -> getPackage(context);
      case LIST -> getList(context);
      default -> throw new IllegalArgumentException("Unsupported asset kind " + assetKind);
    };
    return content.map(HttpResponses::ok).orElse(notFound());
  };

  public final Handler upload = context -> {
    State state = context.getAttributes().require(State.class);
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    checkArgument(PACKAGE == assetKind, "Unsupported asset kind " + assetKind);
    content(context).upload(GolangPathUtils.getAttributesFromMatcherState(state), context.getRequest().getPayload());
    return created();
  };

  private Optional<Content> getList(final Context context) {
    State state = context.getAttributes().require(State.class);
    return content(context).getVersions(GolangPathUtils.module(state));
  }

  private Optional<Content> getPackage(final Context context) {
    State state = context.getAttributes().require(State.class);
    return content(context).getAsset(GolangPathUtils.assetPath(state)).map(FluentAsset::download);
  }

  private Optional<Content> getInfo(final Context context) {
    State state = context.getAttributes().require(State.class);
    return content(context).getInfo(GolangPathUtils.getAttributesFromMatcherState(state));
  }

  private static GoContentFacet content(final Context context) {
    return context.getRepository().facet(GoContentFacet.class);
  }
}
