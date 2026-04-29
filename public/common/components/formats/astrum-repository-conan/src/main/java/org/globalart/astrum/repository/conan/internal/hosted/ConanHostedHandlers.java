
package org.globalart.astrum.repository.conan.internal.hosted;

import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.conan.AssetKind;
import org.globalart.astrum.repository.conan.ConanContentFacet;
import org.globalart.astrum.repository.conan.ConanCoords;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.ConanInfo;
import org.globalart.astrum.repository.conan.internal.ConanJson;
import org.globalart.astrum.repository.conan.internal.ConanPaths;
import org.globalart.astrum.repository.conan.internal.ConanReferencePattern;
import org.globalart.astrum.repository.conan.internal.ConanRevision;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.repository.http.HttpResponses.notFound;

@Component
public class ConanHostedHandlers
{
  private static final String CHECKSUM_DEPLOY = "X-Checksum-Deploy";

  private static final String CHECKSUM_SHA1 = "X-Checksum-Sha1";

  public final Handler handler = context -> {
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    return switch (endpoint) {
      case SEARCH_V1, SEARCH_V2 -> search(context);
      case RECIPE_LATEST -> recipeLatest(context);
      case RECIPE_REVISIONS -> recipeRevisions(context);
      case RECIPE_SEARCH -> packagesSearch(context, false, true);
      case PACKAGES_SEARCH -> packagesSearch(context, false, false);
      case V1_PACKAGES_SEARCH -> packagesSearch(context, true, true);
      case RECIPE_FILES -> recipeFiles(context);
      case PACKAGE_LATEST -> packageLatest(context);
      case PACKAGE_REVISIONS -> packageRevisions(context);
      case PACKAGE_FILES -> packageFiles(context);
      case RECIPE_FILE, PACKAGE_FILE, V1_RECIPE_FILE, V1_PACKAGE_FILE -> download(context);
      case RECIPE_FILE_UPLOAD, PACKAGE_FILE_UPLOAD, V1_RECIPE_FILE_UPLOAD, V1_PACKAGE_FILE_UPLOAD -> upload(context);
      case RECIPE_DELETE, V1_RECIPE_DELETE -> delete(context, ConanPaths.recipeBase(coords(context)) + "/");
      case RECIPE_REVISION_DELETE -> delete(context, ConanPaths.recipeRevisionBase(coords(context)) + "/");
      case PACKAGES_DELETE -> delete(context, ConanPaths.packagesBase(coords(context)));
      case PACKAGE_DELETE -> delete(context, ConanPaths.packageBase(coords(context)) + "/");
      case PACKAGE_REVISION_DELETE -> delete(context, ConanPaths.packageRevisionBase(coords(context)));
      case V1_PACKAGES_DELETE -> v1DeletePackages(context);
      case V1_RECIPE_SNAPSHOT, V1_PACKAGE_SNAPSHOT -> v1Snapshot(context);
      case V1_RECIPE_DIGEST, V1_PACKAGE_DIGEST -> v1Digest(context);
      case V1_RECIPE_DOWNLOAD_URLS, V1_PACKAGE_DOWNLOAD_URLS -> v1DownloadUrls(context);
      case V1_RECIPE_UPLOAD_URLS, V1_PACKAGE_UPLOAD_URLS -> v1UploadUrls(context);
      default -> notFound();
    };
  };

  private static ConanContentFacet content(final Context context) {
    return context.getRepository().facet(ConanContentFacet.class);
  }

  private static ConanCoords coords(final Context context) {
    return ConanCoords.fromState(context.getAttributes().require(TokenMatcher.State.class));
  }

  public static Response search(final Context context) {
    String query = context.getRequest().getParameters().get("q");
    boolean ignoreCase = !"False".equalsIgnoreCase(context.getRequest().getParameters().get("ignorecase"));
    List<String> references = content(context).searchRecipes(ConanReferencePattern.of(query, ignoreCase)).stream()
        .map(ConanCoords::reference)
        .distinct()
        .toList();
    return ConanJson.ok(ConanJson.results(references));
  }

  private Response recipeLatest(final Context context) {
    return content(context).latestRecipeRevision(coords(context))
        .map(revision -> ConanJson.ok(revision.toJson()))
        .orElseGet(HttpResponses::notFound);
  }

  private Response recipeRevisions(final Context context) {
    ConanCoords coords = coords(context);
    List<ConanRevision> revisions = content(context).recipeRevisions(coords);
    if (revisions.isEmpty()) {
      return notFound();
    }
    return ConanJson.ok(ConanJson.revisions(coords.reference(), revisions));
  }

  private Response packageLatest(final Context context) {
    return content(context).latestPackageRevision(coords(context))
        .map(revision -> ConanJson.ok(revision.toJson()))
        .orElseGet(HttpResponses::notFound);
  }

  private Response packageRevisions(final Context context) {
    ConanCoords coords = coords(context);
    List<ConanRevision> revisions = content(context).packageRevisions(coords);
    if (revisions.isEmpty()) {
      return notFound();
    }
    String reference = coords.reference() + "#" + coords.getRecipeRevision() + ":" + coords.getPackageId();
    return ConanJson.ok(ConanJson.revisions(reference, revisions));
  }

  private Response recipeFiles(final Context context) {
    ConanCoords coords = coords(context);
    if (!content(context).recipeRevisionExists(coords)) {
      return notFound();
    }
    return ConanJson.ok(ConanJson.files(content(context).files(ConanPaths.exportBase(coords)).keySet()));
  }

  private Response packageFiles(final Context context) {
    ConanCoords coords = coords(context);
    if (!content(context).packageRevisionExists(coords)) {
      return notFound();
    }
    return ConanJson.ok(ConanJson.files(content(context).files(ConanPaths.packageRevisionBase(coords)).keySet()));
  }

  private Response packagesSearch(final Context context, final boolean v1, final boolean latest) {
    ConanCoords coords = coords(context);
    ConanContentFacet content = content(context);
    if (latest) {
      Optional<ConanRevision> revision = content.latestRecipeRevision(coords);
      if (revision.isEmpty()) {
        return notFound();
      }
      coords = coords.withRecipeRevision(revision.get().revision());
    }
    else if (!content.recipeRevisionExists(coords)) {
      return notFound();
    }
    Map<String, Object> result = new LinkedHashMap<>();
    for (Map.Entry<String, ConanInfo> entry : content.packageInfos(coords).entrySet()) {
      result.put(entry.getKey(), v1 ? entry.getValue().toV1Json() : entry.getValue().toV2Json());
    }
    return ConanJson.ok(result);
  }

  private Response download(final Context context) {
    return content(context).getAsset(ConanPaths.file(coords(context)))
        .map(FluentAsset::download)
        .map(HttpResponses::ok)
        .orElseGet(HttpResponses::notFound);
  }

  private Response upload(final Context context) {
    ConanCoords coords = coords(context);
    String path = ConanPaths.file(coords);
    ConanContentFacet content = content(context);
    if ("true".equalsIgnoreCase(ConanJson.header(context, CHECKSUM_DEPLOY))) {
      String sha1 = ConanJson.header(context, CHECKSUM_SHA1);
      boolean present = sha1 != null && content.getAsset(path)
          .flatMap(FluentAsset::blob)
          .map(blob -> sha1.equalsIgnoreCase(blob.checksums().get(SHA1.name())))
          .orElse(false);
      return present ? HttpResponses.created() : notFound();
    }
    boolean manifest = ConanPaths.isManifest(coords.getFile());
    AssetKind kind = coords.getPackageId() == null
        ? (manifest ? AssetKind.RECIPE_MANIFEST : AssetKind.RECIPE_FILE)
        : (manifest ? AssetKind.PACKAGE_MANIFEST : AssetKind.PACKAGE_FILE);
    content.put(coords, path, kind, context.getRequest().getPayload());
    return HttpResponses.created();
  }

  private Response delete(final Context context, final String prefix) {
    ConanCoords coords = coords(context);
    int deleted = content(context).deleteUnder(coords.recipe(), prefix);
    return deleted > 0 ? HttpResponses.ok() : notFound();
  }

  private Response v1DeletePackages(final Context context) throws IOException {
    ConanCoords coords = coords(context);
    ConanContentFacet content = content(context);
    Optional<ConanRevision> revision = content.latestRecipeRevision(coords);
    if (revision.isEmpty()) {
      return notFound();
    }
    ConanCoords recipe = coords.withRecipeRevision(revision.get().revision());
    List<String> packageIds = new ArrayList<>();
    if (context.getRequest().getPayload() != null) {
      Object ids = ConanJson.read(context.getRequest().getPayload()).get("package_ids");
      if (ids instanceof List<?> list) {
        list.forEach(id -> packageIds.add(String.valueOf(id)));
      }
    }
    if (packageIds.isEmpty()) {
      content.deleteUnder(recipe.recipe(), ConanPaths.packagesBase(recipe));
    }
    else {
      for (String packageId : packageIds) {
        ConanCoords pkg = recipe.withPackage(packageId, null);
        pkg.validate();
        content.deleteUnder(recipe.recipe(), ConanPaths.packageBase(pkg) + "/");
      }
    }
    return HttpResponses.ok();
  }

  private Optional<ConanCoords> v1Latest(final Context context) {
    ConanCoords coords = coords(context);
    ConanContentFacet content = content(context);
    Optional<ConanRevision> recipeRevision = content.latestRecipeRevision(coords);
    if (recipeRevision.isEmpty()) {
      return Optional.empty();
    }
    ConanCoords latest = coords.withRecipeRevision(recipeRevision.get().revision());
    if (latest.getPackageId() == null) {
      return Optional.of(latest);
    }
    return content.latestPackageRevision(latest).map(revision -> latest.withPackageRevision(revision.revision()));
  }

  private static String directory(final ConanCoords coords) {
    return coords.getPackageId() == null ? ConanPaths.exportBase(coords) : ConanPaths.packageRevisionBase(coords);
  }

  private static String v1FileUrl(final Context context, final ConanCoords coords, final String file) {
    StringBuilder url = new StringBuilder(context.getRepository().getUrl())
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

  private Map<String, FluentAsset> v1Files(final Context context, final ConanCoords coords) {
    Map<String, FluentAsset> files = new LinkedHashMap<>();
    content(context).files(directory(coords)).forEach((name, asset) -> {
      if (!name.contains("/")) {
        files.put(name, asset);
      }
    });
    return files;
  }

  private Response v1Snapshot(final Context context) {
    Optional<ConanCoords> latest = v1Latest(context);
    if (latest.isEmpty()) {
      return notFound();
    }
    Map<String, Object> snapshot = new LinkedHashMap<>();
    v1Files(context, latest.get()).forEach((name, asset) -> asset.blob()
        .map(blob -> blob.checksums().get(MD5.name()))
        .ifPresent(md5 -> snapshot.put(name, md5)));
    if (snapshot.isEmpty()) {
      return notFound();
    }
    return ConanJson.ok(snapshot);
  }

  private Response v1Digest(final Context context) {
    Optional<ConanCoords> latest = v1Latest(context);
    if (latest.isEmpty()) {
      return notFound();
    }
    return ConanJson.ok(Map.of(ConanPaths.CONAN_MANIFEST, v1FileUrl(context, latest.get(), ConanPaths.CONAN_MANIFEST)));
  }

  private Response v1DownloadUrls(final Context context) {
    Optional<ConanCoords> latest = v1Latest(context);
    if (latest.isEmpty()) {
      return notFound();
    }
    Map<String, Object> urls = new LinkedHashMap<>();
    v1Files(context, latest.get()).keySet().forEach(name -> urls.put(name, v1FileUrl(context, latest.get(), name)));
    if (urls.isEmpty()) {
      return notFound();
    }
    return ConanJson.ok(urls);
  }

  private Response v1UploadUrls(final Context context) throws IOException {
    ConanCoords coords = coords(context);
    if (context.getRequest().getPayload() == null) {
      return HttpResponses.badRequest("Missing file list");
    }
    Map<String, Object> requested = ConanJson.read(context.getRequest().getPayload());
    if (requested.isEmpty()) {
      return HttpResponses.badRequest("Missing file list");
    }
    ConanCoords target;
    if (coords.getPackageId() == null) {
      target = coords.withRecipeRevision(ConanPaths.V1_REVISION);
    }
    else {
      String recipeRevision = content(context).latestRecipeRevision(coords)
          .map(ConanRevision::revision)
          .orElse(ConanPaths.V1_REVISION);
      target = coords.withRecipeRevision(recipeRevision).withPackageRevision(ConanPaths.V1_REVISION);
    }
    Map<String, Object> urls = new LinkedHashMap<>();
    for (String name : requested.keySet()) {
      target.withFile(name).validate();
      urls.put(name, v1FileUrl(context, target, name));
    }
    return ConanJson.ok(urls);
  }
}
