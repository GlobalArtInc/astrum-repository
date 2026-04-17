
package org.globalart.astrum.repository.conda.util;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import static com.google.common.base.Preconditions.checkNotNull;

public final class CondaPathUtils
{
  public static final String REPODATA_JSON = "repodata.json";

  public static final String REPODATA_JSON_ZST = "repodata.json.zst";

  public static final String TAR_BZ2_EXT = ".tar.bz2";

  public static final String CONDA_EXT = ".conda";

  public static final String PATH_TOKEN = "path";

  public static final String SUBDIR_TOKEN = "subdir";

  public static final String NAME_TOKEN = "name";

  public static final String VERSION_TOKEN = "version";

  public static final String BUILD_TOKEN = "build";

  private static final String CHANNEL_PREFIX = "{path:.*}/";

  private static final String SUBDIR_PREFIX = CHANNEL_PREFIX + "{subdir:[^/]+}/";

  private static final String PACKAGE_PREFIX = SUBDIR_PREFIX + "{name:[^/]+}-{version:[^/-]+}-{build:[^/-]+}";

  private static final Pattern PACKAGE_PATH = Pattern.compile(
      "^(?<path>.*)/(?<subdir>[^/]+)/(?<name>[^/]+)-(?<version>[^/-]+)-(?<build>[^/-]+)(?<ext>\\.tar\\.bz2|\\.conda)$");

  private static final Pattern SUBDIR_NAME = Pattern.compile("noarch|[a-z][a-z0-9]*-[a-z0-9_]+");

  private CondaPathUtils() {
  }

  public static String pattern(final AssetKind assetKind) {
    switch (assetKind) {
      case ARCH_TAR_PACKAGE:
        return PACKAGE_PREFIX + TAR_BZ2_EXT;
      case ARCH_CONDA_PACKAGE:
        return PACKAGE_PREFIX + CONDA_EXT;
      case ARCH_SHARD:
        return SUBDIR_PREFIX + "{shard:[0-9a-f]+}.msgpack.zst";
      default:
        return (assetKind.isSubdirLevel() ? SUBDIR_PREFIX : CHANNEL_PREFIX) + assetKind.getFileName();
    }
  }

  public static State matcherState(final Context context) {
    return context.getAttributes().require(State.class);
  }

  public static String subdir(final State state) {
    return match(state, SUBDIR_TOKEN);
  }

  public static String name(final State state) {
    return match(state, NAME_TOKEN);
  }

  public static String version(final State state) {
    return match(state, VERSION_TOKEN);
  }

  public static String channelPath(final State state) {
    return match(state, PATH_TOKEN);
  }

  public static String subdirPath(final State state) {
    return normalizeAssetPath(channelPath(state) + "/" + subdir(state));
  }

  public static String repodataPath(final String subdirPath) {
    return subdirPath + "/" + REPODATA_JSON;
  }

  private static String match(final State state, final String name) {
    checkNotNull(state);
    String result = state.getTokens().get(name);
    checkNotNull(result);
    return result;
  }

  public static String assetPath(final Context context) {
    return normalizeAssetPath(context.getRequest().getPath());
  }

  public static String normalizeAssetPath(final String path) {
    return path.startsWith("/") ? path : "/" + path;
  }

  public static boolean isValidSubdir(@Nullable final String subdir) {
    return subdir != null && SUBDIR_NAME.matcher(subdir).matches();
  }

  public static String lastSegment(final String path) {
    return path.substring(path.lastIndexOf('/') + 1);
  }

  public static Optional<PackagePath> parsePackagePath(final String path) {
    Matcher matcher = PACKAGE_PATH.matcher(normalizeAssetPath(path));
    if (!matcher.matches()) {
      return Optional.empty();
    }
    return Optional.of(new PackagePath(
        normalizeAssetPath(matcher.group(PATH_TOKEN) + "/" + matcher.group(SUBDIR_TOKEN)),
        matcher.group(SUBDIR_TOKEN),
        matcher.group(NAME_TOKEN),
        matcher.group(VERSION_TOKEN),
        matcher.group(BUILD_TOKEN),
        CONDA_EXT.equals(matcher.group("ext")) ? AssetKind.ARCH_CONDA_PACKAGE : AssetKind.ARCH_TAR_PACKAGE));
  }

  public static AssetKind packageKind(final String fileName) {
    if (fileName.endsWith(CONDA_EXT)) {
      return AssetKind.ARCH_CONDA_PACKAGE;
    }
    if (fileName.endsWith(TAR_BZ2_EXT)) {
      return AssetKind.ARCH_TAR_PACKAGE;
    }
    throw new IllegalArgumentException("Conda packages must end with " + TAR_BZ2_EXT + " or " + CONDA_EXT);
  }

  public static String extension(final AssetKind assetKind) {
    return assetKind == AssetKind.ARCH_CONDA_PACKAGE ? CONDA_EXT : TAR_BZ2_EXT;
  }

  public static final class PackagePath
  {
    private final String subdirPath;

    private final String subdir;

    private final String name;

    private final String version;

    private final String build;

    private final AssetKind assetKind;

    public PackagePath(
        final String subdirPath,
        final String subdir,
        final String name,
        final String version,
        final String build,
        final AssetKind assetKind)
    {
      this.subdirPath = subdirPath;
      this.subdir = subdir;
      this.name = name;
      this.version = version;
      this.build = build;
      this.assetKind = assetKind;
    }

    public String getSubdirPath() {
      return subdirPath;
    }

    public String getSubdir() {
      return subdir;
    }

    public String getName() {
      return name;
    }

    public String getVersion() {
      return version;
    }

    public String getBuild() {
      return build;
    }

    public AssetKind getAssetKind() {
      return assetKind;
    }

    public String getFileName() {
      return name + "-" + version + "-" + build + extension(assetKind);
    }

    public String getPath() {
      return subdirPath + "/" + getFileName();
    }
  }
}
