package org.globalart.astrum.repository.terraform.internal.hosted;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformRequest;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.ModulePackages;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;

import org.springframework.stereotype.Component;

@Component
public class TerraformHostedHandlers
{
  public final Handler moduleVersions = context -> {
    ModuleCoordinates module = TerraformRequest.of(context).module();
    List<String> versions = hosted(context).moduleVersions(module);
    return versions.isEmpty()
        ? TerraformResponses.error(404, "Module " + module.address() + " not found")
        : TerraformResponses.json(TerraformResponses.moduleVersions(versions));
  };

  public final Handler moduleDownload = context -> {
    ModuleCoordinates module = TerraformRequest.of(context).module();
    return hosted(context).moduleArchive(module)
        .map(archive -> TerraformResponses.location(DownloadTokens.of(content(context))
            .sign(ModulePackages.location(module, archive), archive.path())))
        .orElseGet(() -> TerraformResponses.error(404, "Module " + module.address() + " version "
            + module.version() + " not found"));
  };

  public final Handler getContent = context -> content(context)
      .get(TerraformRequest.of(context).assetPath())
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler putModule = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    hosted(context).putModule(request.module(), request.file(), context.getRequest().getPayload(), path -> { });
    return HttpResponses.created();
  };

  public final Handler putProviderFile = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    hosted(context).putProviderFile(request.provider(), request.file(), context.getRequest().getPayload(),
        TerraformUploadHandler.protocols(context.getRequest().getParameters().get("protocols")), path -> { });
    return HttpResponses.created();
  };

  public final Handler delete = context -> {
    Optional<FluentAsset> asset = content(context).getAsset(TerraformRequest.of(context).assetPath());
    if (asset.isEmpty()) {
      return HttpResponses.notFound();
    }
    context.getRepository().facet(ContentMaintenanceFacet.class).deleteAsset(asset.get());
    return HttpResponses.noContent();
  };

  public final Handler providerVersions = context -> {
    ProviderCoordinates provider = TerraformRequest.of(context).provider();
    return hosted(context).providerVersions(provider)
        .map(TerraformResponses::json)
        .orElseGet(() -> TerraformResponses.error(404, "Provider " + provider.address() + " not found"));
  };

  public final Handler providerDownload = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    TerraformHostedFacet.ProviderPackage found = hosted(context).providerPackage(request.provider(),
        request.platform());
    return found.document() != null
        ? TerraformResponses.json(found.document())
        : TerraformResponses.error(found.status(), found.reason());
  };

  public final Handler mirrorIndex = context -> {
    ProviderCoordinates provider = TerraformRequest.of(context).provider();
    return hosted(context).mirrorIndex(provider)
        .map(TerraformResponses::json)
        .orElseGet(() -> TerraformResponses.error(404, "Provider " + provider.address() + " not found"));
  };

  public final Handler mirrorVersion = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    ProviderCoordinates provider = request.provider();
    return hosted(context).mirrorVersion(provider, context.getRequest().getPath())
        .map(TerraformResponses::json)
        .orElseGet(() -> TerraformResponses.error(404, "Provider " + provider.address() + " version "
            + provider.version() + " not found"));
  };


  private static TerraformHostedFacet hosted(final Context context) {
    return context.getRepository().facet(TerraformHostedFacet.class);
  }

  private static TerraformContentFacet content(final Context context) {
    return context.getRepository().facet(TerraformContentFacet.class);
  }
}
