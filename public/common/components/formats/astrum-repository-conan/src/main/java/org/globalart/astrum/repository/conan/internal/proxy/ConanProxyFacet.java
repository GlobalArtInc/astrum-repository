
package org.globalart.astrum.repository.conan.internal.proxy;

import java.io.IOException;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.conan.AssetKind;
import org.globalart.astrum.repository.conan.ConanContentFacet;
import org.globalart.astrum.repository.conan.ConanCoords;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.ConanJson;
import org.globalart.astrum.repository.conan.internal.ConanPaths;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ConanProxyFacet
    extends ContentProxyFacetSupport
{
  public static final String CONFIG_KEY = "conan";

  public static final String CONAN_VERSION = "conanVersion";

  public static final String V1 = "V1";

  public static final String V2 = "V2";

  public static class Config
  {
    public String conanVersion = V2;
  }

  private Config conanConfig = new Config();

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    super.doConfigure(configuration);
    Config config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, Config.class);
    conanConfig = config != null ? config : new Config();
  }

  public static String normalizeVersion(@Nullable final String version) {
    return version != null && V1.equals(version.trim().toUpperCase(Locale.ROOT)) ? V1 : V2;
  }

  public String conanVersion() {
    return normalizeVersion(conanConfig.conanVersion);
  }

  public boolean isV1() {
    return V1.equals(conanVersion());
  }

  public boolean supports(final ConanEndpoint endpoint) {
    return endpoint.protocol() == 0 || endpoint.protocol() == (isV1() ? 1 : 2);
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    ConanCoords coords = coords(context);
    Optional<FluentAsset> asset = content().getAsset(storagePath(endpoint, coords));
    if (asset.isEmpty()) {
      return null;
    }
    Content cached = asset.get().download();
    return isUrlIndex(endpoint) ? rewrite(cached, coords) : cached;
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    ConanCoords coords = coords(context);
    String path = storagePath(endpoint, coords);
    AssetKind kind = kind(endpoint, coords);
    try (TempBlob blob = content().ingest(content)) {
      if (isUrlIndex(endpoint)) {
        invalidateIfChanged(coords, path, blob);
      }
      FluentAsset asset = content().put(coords, path, kind, blob);
      Content stored = asset.markAsCached(content).download();
      return isUrlIndex(endpoint) ? rewrite(stored, coords) : stored;
    }
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    AssetKind kind = context.getAttributes().require(AssetKind.class);
    return cacheControllerHolder.require(kind.getCacheType());
  }

  @Nullable
  @Override
  protected String getUrl(@Nonnull final Context context) {
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    if (endpoint == ConanEndpoint.V1_RECIPE_FILE || endpoint == ConanEndpoint.V1_PACKAGE_FILE) {
      return v1FileUrl(coords(context));
    }
    return context.getRequest().getPath().substring(1);
  }

  private static boolean isUrlIndex(final ConanEndpoint endpoint) {
    return switch (endpoint) {
      case V1_RECIPE_DOWNLOAD_URLS, V1_PACKAGE_DOWNLOAD_URLS, V1_RECIPE_DIGEST, V1_PACKAGE_DIGEST -> true;
      default -> false;
    };
  }

  private static AssetKind kind(final ConanEndpoint endpoint, final ConanCoords coords) {
    AssetKind kind = endpoint.assetKind();
    if (kind == null) {
      throw new IllegalStateException("Endpoint " + endpoint + " is not cacheable");
    }
    if (coords.getFile() != null && ConanPaths.isManifest(coords.getFile())) {
      return kind == AssetKind.PACKAGE_FILE ? AssetKind.PACKAGE_MANIFEST : AssetKind.RECIPE_MANIFEST;
    }
    return kind;
  }

  private static String storagePath(final ConanEndpoint endpoint, final ConanCoords coords) {
    return switch (endpoint) {
      case RECIPE_FILE, PACKAGE_FILE, V1_RECIPE_FILE, V1_PACKAGE_FILE -> ConanPaths.file(coords);
      default -> ConanPaths.metadata(endpoint, coords);
    };
  }

  @Nullable
  private String v1FileUrl(final ConanCoords coords) {
    String file = coords.getFile();
    boolean recipe = coords.getPackageId() == null;
    ConanEndpoint index = recipe ? ConanEndpoint.V1_RECIPE_DOWNLOAD_URLS : ConanEndpoint.V1_PACKAGE_DOWNLOAD_URLS;
    String url = lookup(ConanPaths.metadata(index, coords), file);
    if (url == null && ConanPaths.isManifest(file)) {
      ConanEndpoint digest = recipe ? ConanEndpoint.V1_RECIPE_DIGEST : ConanEndpoint.V1_PACKAGE_DIGEST;
      url = lookup(ConanPaths.metadata(digest, coords), file);
    }
    if (url == null) {
      log.debug("No upstream URL known for {} in {}", coords, getRepository().getName());
    }
    return url;
  }

  @Nullable
  private String lookup(final String indexPath, final String file) {
    return content().getAsset(indexPath)
        .flatMap(asset -> {
          try {
            Object url = ConanJson.read(asset.download().getPayload()).get(file);
            return Optional.ofNullable(url instanceof String text ? text : null);
          }
          catch (IOException e) {
            log.warn("Unable to read {} in {}: {}", indexPath, getRepository().getName(), e.getMessage());
            return Optional.empty();
          }
        })
        .orElse(null);
  }

  private Content rewrite(final Content content, final ConanCoords coords) throws IOException {
    Map<String, Object> upstream = ConanJson.read(content.getPayload());
    ConanCoords local = ConanPaths.v1(coords);
    Map<String, Object> rewritten = new LinkedHashMap<>();
    for (String file : upstream.keySet()) {
      local.withFile(file).validate();
      rewritten.put(file, localUrl(local, file));
    }
    return new Content(content, new BytesPayload(ConanJson.bytes(rewritten), APPLICATION_JSON));
  }

  private String localUrl(final ConanCoords coords, final String file) {
    StringBuilder url = new StringBuilder(getRepository().getUrl())
        .append("/v1/files")
        .append(ConanPaths.recipeRevisionBase(coords));
    if (coords.getPackageId() == null) {
      url.append('/').append(ConanPaths.EXPORT);
    }
    else {
      url.append('/').append(ConanPaths.PACKAGE).append('/').append(coords.getPackageId())
          .append('/').append(coords.getPackageRevision());
    }
    return url.append('/').append(file).toString();
  }

  private void invalidateIfChanged(final ConanCoords coords, final String path, final TempBlob blob) {
    String sha1 = blob.getHashes().get(SHA1).toString();
    boolean changed = content().getAsset(path)
        .flatMap(FluentAsset::blob)
        .map(existing -> !Objects.equals(existing.checksums().get(SHA1.name()), sha1))
        .orElse(false);
    if (!changed) {
      return;
    }
    ConanCoords local = ConanPaths.v1(coords);
    String directory = local.getPackageId() == null
        ? ConanPaths.exportBase(local)
        : ConanPaths.packageRevisionBase(local);
    content().files(directory).values().forEach(FluentAsset::delete);
    log.debug("Upstream index {} changed in {}, dropped cached files under {}", path, getRepository().getName(),
        directory);
  }

  private static ConanCoords coords(final Context context) {
    return ConanCoords.fromState(context.getAttributes().require(TokenMatcher.State.class));
  }

  private ConanContentFacet content() {
    return facet(ConanContentFacet.class);
  }
}
