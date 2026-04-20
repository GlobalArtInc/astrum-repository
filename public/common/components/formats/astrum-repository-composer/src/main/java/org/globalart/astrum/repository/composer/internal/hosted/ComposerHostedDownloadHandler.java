
package org.globalart.astrum.repository.composer.internal.hosted;

import org.springframework.stereotype.Component;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildZipballPath;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.getProjectToken;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.getVendorToken;

/**
 * Download handler for Composer hosted repositories.
 */
@Component
public class ComposerHostedDownloadHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Repository repository = context.getRepository();
    ComposerHostedFacet hostedFacet = repository.facet(ComposerHostedFacet.class);
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    switch (assetKind) {
      case PACKAGES:
        return HttpResponses.ok(hostedFacet.getPackagesJson());
      case LIST:
        return responseFor(hostedFacet.getListJson(context.getRequest().getParameters().get("filter")));
      case PROVIDER:
        return responseFor(hostedFacet.getProviderJson(getVendorToken(context), getProjectToken(context)));
      case PACKAGE:
        return responseFor(hostedFacet.getPackageJson(getVendorToken(context), getProjectToken(context)));
      case ZIPBALL:
        return responseFor(hostedFacet.getZipball(buildZipballPath(context)));
      default:
        throw new IllegalStateException("Unexpected assetKind: " + assetKind);
    }
  }

  private Response responseFor(@Nullable final Content content) {
    if (content == null) {
      return HttpResponses.notFound();
    }
    return HttpResponses.ok(content);
  }
}
