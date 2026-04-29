package org.globalart.astrum.repository.conan.internal;

import org.globalart.astrum.repository.conan.ConanCoords;

import static com.google.common.base.Preconditions.checkNotNull;

public final class ConanPaths
{
  public static final String V1_REVISION = "0";

  public static final String EXPORT = "export";

  public static final String PACKAGE = "package";

  public static final String CONAN_MANIFEST = "conanmanifest.txt";

  public static final String CONAN_INFO = "conaninfo.txt";

  public static final String CONAN_FILE = "conanfile.py";

  public static final String CONAN_SOURCES = "conan_sources.tgz";

  public static final String METADATA_PREFIX = "metadata/";

  private ConanPaths() {
  }

  public static String recipeBase(final ConanCoords coords) {
    return "/" + coords.getName() + "/" + coords.getVersion() + "/" + coords.getUser() + "/" + coords.getChannel();
  }

  public static String recipeRevisionBase(final ConanCoords coords) {
    return recipeBase(coords) + "/" + checkNotNull(coords.getRecipeRevision());
  }

  public static String exportBase(final ConanCoords coords) {
    return recipeRevisionBase(coords) + "/" + EXPORT + "/";
  }

  public static String recipeFile(final ConanCoords coords, final String file) {
    return exportBase(coords) + file;
  }

  public static String packagesBase(final ConanCoords coords) {
    return recipeRevisionBase(coords) + "/" + PACKAGE + "/";
  }

  public static String packageBase(final ConanCoords coords) {
    return packagesBase(coords) + checkNotNull(coords.getPackageId());
  }

  public static String packageRevisionBase(final ConanCoords coords) {
    return packageBase(coords) + "/" + checkNotNull(coords.getPackageRevision()) + "/";
  }

  public static String packageFile(final ConanCoords coords, final String file) {
    return packageRevisionBase(coords) + file;
  }

  public static String file(final ConanCoords coords) {
    String file = checkNotNull(coords.getFile());
    return coords.getPackageId() == null ? recipeFile(coords, file) : packageFile(coords, file);
  }

  public static boolean isManifest(final String file) {
    return CONAN_MANIFEST.equals(file);
  }

  public static boolean isMetadata(final String path) {
    return path.contains("/" + METADATA_PREFIX);
  }

  public static String metadata(final ConanEndpoint endpoint, final ConanCoords coords) {
    return switch (endpoint) {
      case RECIPE_LATEST -> recipeBase(coords) + "/latest.json";
      case RECIPE_REVISIONS -> recipeBase(coords) + "/revisions.json";
      case RECIPE_SEARCH -> recipeBase(coords) + "/search.json";
      case V1_PACKAGES_SEARCH -> recipeBase(coords) + "/search-v1.json";
      case RECIPE_FILES -> recipeRevisionBase(coords) + "/files.json";
      case PACKAGES_SEARCH -> recipeRevisionBase(coords) + "/search.json";
      case PACKAGE_LATEST -> packageBase(coords) + "/latest.json";
      case PACKAGE_REVISIONS -> packageBase(coords) + "/revisions.json";
      case PACKAGE_FILES -> packageRevisionBase(coords) + "files.json";
      case V1_RECIPE_SNAPSHOT -> recipeRevisionBase(v1(coords)) + "/snapshot.json";
      case V1_RECIPE_DIGEST -> recipeRevisionBase(v1(coords)) + "/digest.json";
      case V1_RECIPE_DOWNLOAD_URLS -> recipeRevisionBase(v1(coords)) + "/download_urls.json";
      case V1_PACKAGE_SNAPSHOT -> packageRevisionBase(v1(coords)) + "snapshot.json";
      case V1_PACKAGE_DIGEST -> packageRevisionBase(v1(coords)) + "digest.json";
      case V1_PACKAGE_DOWNLOAD_URLS -> packageRevisionBase(v1(coords)) + "download_urls.json";
      default -> throw new IllegalArgumentException("No metadata path for " + endpoint);
    };
  }

  public static ConanCoords v1(final ConanCoords coords) {
    ConanCoords v1 = coords.withRecipeRevision(V1_REVISION);
    return coords.getPackageId() == null ? v1 : v1.withPackageRevision(V1_REVISION);
  }

  public static String escapeLike(final String value) {
    StringBuilder escaped = new StringBuilder(value.length() + 8);
    for (char c : value.toCharArray()) {
      if (c == '\\' || c == '%' || c == '_') {
        escaped.append('\\');
      }
      escaped.append(c);
    }
    return escaped.toString();
  }
}
