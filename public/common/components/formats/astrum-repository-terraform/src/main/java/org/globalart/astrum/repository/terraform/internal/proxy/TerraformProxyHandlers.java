package org.globalart.astrum.repository.terraform.internal.proxy;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.TreeMap;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.terraform.AssetKind;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.Platform;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.ProviderFiles;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.TerraformRequest;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.ModulePackages;
import org.globalart.astrum.repository.terraform.internal.SemanticVersions;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.terraform.internal.signing.Pgp;
import org.globalart.astrum.repository.terraform.internal.signing.Sha256Sums;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.stereotype.Component;

@Component
public class TerraformProxyHandlers
{
  /**
   * Points Terraform at the cached package when the upstream package can be cached, else at the upstream location.
   */
  public final Handler moduleDownload = context -> {
    ModuleCoordinates module = TerraformRequest.of(context).module();
    Optional<FluentAsset> cached = content(context).assetsIn(module.directory(), AssetKind.MODULE_ARCHIVE).stream()
        .findFirst();
    DownloadTokens tokens = DownloadTokens.of(content(context));
    if (cached.isPresent()) {
      return TerraformResponses.location(tokens.sign(ModulePackages.location(module, cached.get()),
          cached.get().path()));
    }
    Optional<String> location = proxy(context).moduleLocation(module);
    if (location.isEmpty()) {
      return TerraformResponses.error(404, "Module " + module.address() + " version " + module.version()
          + " not found");
    }
    ModuleSource source = ModuleSource.parse(location.get());
    if (source.cacheable()) {
      Optional<FluentAsset> fetched = proxy(context).cacheModule(context, module, source);
      if (fetched.isPresent()) {
        return TerraformResponses.location(tokens.sign(ModulePackages.location(module, fetched.get()),
            fetched.get().path()));
      }
    }
    return TerraformResponses.location(localLocation(module, location.get(), tokens));
  };

  public final Handler providerDownload = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    Optional<JsonNode> download = proxy(context).providerDownload(context, request.provider(), request.platform());
    if (download.isEmpty()) {
      return TerraformResponses.error(404, "Provider " + request.provider().address() + " version "
          + request.provider().version() + " has no package for " + request.platform());
    }
    URI endpoint = proxy(context).downloadEndpoint(request.provider(), request.platform());
    return TerraformResponses.json(rewrite(download.get(), request.provider(), request.platform(), endpoint,
        DownloadTokens.of(content(context))));
  };

  public final Handler mirrorIndex = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    TerraformProxyFacet proxy = proxy(context);
    if (!proxy.upstreamHost().equals(request.hostname())) {
      return notMirrored(request.hostname(), proxy);
    }
    Optional<JsonNode> versions = proxy.providerVersions(context, request.provider());
    if (versions.isEmpty()) {
      return TerraformResponses.error(404, "Provider " + request.provider().address() + " not found");
    }
    List<String> list = new ArrayList<>();
    versions.get().path("versions").forEach(entry -> {
      String version = entry.path("version").asText();
      if (TerraformPaths.isValidVersion(version)) {
        list.add(version);
      }
    });
    list.sort(SemanticVersions.ORDER);
    return TerraformResponses.json(TerraformResponses.mirrorIndex(list));
  };

  /**
   * The packages of a provider version for the network mirror protocol. The hashes come from the upstream
   * checksums, which are only trusted once their signature verifies with a key the upstream registry lists.
   */
  public final Handler mirrorVersion = context -> {
    TerraformRequest request = TerraformRequest.of(context);
    TerraformProxyFacet proxy = proxy(context);
    if (!proxy.upstreamHost().equals(request.hostname())) {
      return notMirrored(request.hostname(), proxy);
    }
    ProviderCoordinates provider = request.provider();
    Optional<JsonNode> download = proxy.anyProviderDownload(context, provider);
    if (download.isEmpty()) {
      return TerraformResponses.error(404, "Provider " + provider.address() + " version "
          + provider.version() + " not found");
    }
    String sumsName = TerraformProxyFacet.baseName(download.get().path("shasums_url").asText());
    String signatureName = TerraformProxyFacet.baseName(download.get().path("shasums_signature_url").asText());
    if (ProviderFiles.kindOf(provider, sumsName).orElse(null) != AssetKind.PROVIDER_SHASUMS
        || ProviderFiles.kindOf(provider, signatureName).orElse(null) != AssetKind.PROVIDER_SIGNATURE) {
      return TerraformResponses.error(502, "The upstream registry names checksums the mirror cannot serve");
    }
    Optional<byte[]> sums = proxy.releaseFile(context, provider, sumsName, AssetKind.PROVIDER_SHASUMS);
    Optional<byte[]> signature = proxy.releaseFile(context, provider, signatureName, AssetKind.PROVIDER_SIGNATURE);
    if (sums.isEmpty() || signature.isEmpty()) {
      return TerraformResponses.error(502, "The checksums of " + provider.address() + " "
          + provider.version() + " are not available upstream");
    }
    if (!signedByListedKey(sums.get(), signature.get(), download.get())) {
      return TerraformResponses.error(502, "The checksums of " + provider.address() + " "
          + provider.version() + " are not signed by a key the upstream registry lists");
    }
    ObjectNode document = TerraformResponses.object();
    ObjectNode archives = document.putObject("archives");
    TreeMap<Platform, String> byPlatform = new TreeMap<>();
    Sha256Sums.parse(sums.get()).digests().forEach((file, digest) ->
        ProviderFiles.platformOf(provider, file).ifPresent(platform -> byPlatform.put(platform, file + " " + digest)));
    String documentPath = context.getRequest().getPath();
    DownloadTokens tokens = DownloadTokens.of(content(context));
    byPlatform.forEach((platform, entry) -> {
      String file = entry.substring(0, entry.indexOf(' '));
      String digest = entry.substring(entry.indexOf(' ') + 1);
      ObjectNode archive = archives.putObject(platform.toString());
      archive.put("url", tokens.sign(TerraformPaths.relative(documentPath, provider.filePath(file)),
          provider.filePath(file)));
      archive.putArray("hashes").add("zh:" + digest);
    });
    return TerraformResponses.json(document);
  };

  /**
   * The upstream package description with the locations of the release files moved into this repository, so they
   * are cached here. A file whose name does not follow the release conventions keeps its upstream location.
   */
  static ObjectNode rewrite(
      final JsonNode upstream,
      final ProviderCoordinates provider,
      final Platform platform,
      final URI endpoint,
      final DownloadTokens tokens)
  {
    ObjectNode document = upstream.deepCopy();
    String from = provider.downloadPath(platform);
    String filename = upstream.path("filename").asText();
    if (provider.archiveFileName(platform).equals(filename)) {
      document.put("download_url", tokens.sign(TerraformPaths.relative(from, provider.filePath(filename)),
          provider.filePath(filename)));
    }
    else {
      absolute(document, "download_url", endpoint);
    }
    rewriteFile(document, "shasums_url", provider, AssetKind.PROVIDER_SHASUMS, from, endpoint, tokens);
    rewriteFile(document, "shasums_signature_url", provider, AssetKind.PROVIDER_SIGNATURE, from, endpoint, tokens);
    return document;
  }

  /**
   * The location of a module package for Terraform: the cached copy when the package can be cached here.
   */
  static String localLocation(final ModuleCoordinates module, final String upstreamLocation,
                              final DownloadTokens tokens)
  {
    ModuleSource source = ModuleSource.parse(upstreamLocation);
    if (!source.cacheable()) {
      return upstreamLocation;
    }
    String archive = module.archivePath(source.extension());
    return tokens.sign(TerraformPaths.relative(module.directory() + "/download", archive) + source.subdirSuffix(),
        archive);
  }


  private static void rewriteFile(
      final ObjectNode document,
      final String field,
      final ProviderCoordinates provider,
      final AssetKind kind,
      final String from,
      final URI endpoint,
      final DownloadTokens tokens)
  {
    String url = document.path(field).asText();
    if (url.isEmpty()) {
      return;
    }
    String name = TerraformProxyFacet.baseName(url);
    if (ProviderFiles.kindOf(provider, name).orElse(null) == kind) {
      document.put(field, tokens.sign(TerraformPaths.relative(from, provider.filePath(name)),
          provider.filePath(name)));
    }
    else {
      absolute(document, field, endpoint);
    }
  }

  private static void absolute(final ObjectNode document, final String field, final URI endpoint) {
    String url = document.path(field).asText();
    if (!url.isEmpty()) {
      document.put(field, endpoint.resolve(url).toString());
    }
  }

  private static boolean signedByListedKey(final byte[] sums, final byte[] signature, final JsonNode download) {
    byte[] binary;
    try {
      binary = Pgp.binarySignature(signature);
    }
    catch (IllegalArgumentException e) {
      return false;
    }
    for (JsonNode key : download.path("signing_keys").path("gpg_public_keys")) {
      String armored = key.path("ascii_armor").asText();
      if (!armored.isEmpty() && Pgp.verify(sums, binary, armored)) {
        return true;
      }
    }
    return false;
  }

  private static Response notMirrored(
      @Nullable final String hostname,
      final TerraformProxyFacet proxy)
  {
    return TerraformResponses.error(404, "This repository mirrors providers of " + proxy.upstreamHost()
        + ", not of " + hostname);
  }

  private static TerraformProxyFacet proxy(final Context context) {
    return context.getRepository().facet(TerraformProxyFacet.class);
  }

  private static TerraformContentFacet content(final Context context) {
    return context.getRepository().facet(TerraformContentFacet.class);
  }
}
