
package org.globalart.astrum.repository.p2.internal.util;

import java.util.LinkedHashMap;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.p2.internal.AssetKind;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.p2.internal.AssetKind.ARTIFACTS_METADATA;
import static org.globalart.astrum.repository.p2.internal.AssetKind.BINARY_BUNDLE;
import static org.globalart.astrum.repository.p2.internal.AssetKind.BUNDLE;
import static org.globalart.astrum.repository.p2.internal.AssetKind.COMPOSITE_ARTIFACTS;
import static org.globalart.astrum.repository.p2.internal.AssetKind.COMPOSITE_CONTENT;
import static org.globalart.astrum.repository.p2.internal.AssetKind.CONTENT_METADATA;
import static org.globalart.astrum.repository.p2.internal.AssetKind.P2_INDEX;

public final class P2PathUtils
{
  public static final String PLUGIN_NAME = "pluginName";

  public static final String SITE = "site";

  public static final String SITE_PREFIX = "/{" + SITE + ":[0-9a-f]{64\\}}";

  private static final String NAME_VERSION =
      "{name:.*}_{version:\\\\d+\\\\.\\\\d+\\\\.\\\\d+(\\\\.[A-Za-z0-9_-]+)?}";

  private static final String EXTENSION = "extension";

  private P2PathUtils() {
    throw new UnsupportedOperationException();
  }

  public static Map<String, AssetKind> patterns(final String prefix) {
    Map<String, AssetKind> patterns = new LinkedHashMap<>();
    patterns.put(prefix + "/p2.index", P2_INDEX);
    patterns.put(prefix + "/compositeContent.jar", COMPOSITE_CONTENT);
    patterns.put(prefix + "/compositeContent.xml", COMPOSITE_CONTENT);
    patterns.put(prefix + "/content.jar", CONTENT_METADATA);
    patterns.put(prefix + "/content.xml.xz", CONTENT_METADATA);
    patterns.put(prefix + "/content.xml", CONTENT_METADATA);
    patterns.put(prefix + "/compositeArtifacts.jar", COMPOSITE_ARTIFACTS);
    patterns.put(prefix + "/compositeArtifacts.xml", COMPOSITE_ARTIFACTS);
    patterns.put(prefix + "/artifacts.jar", ARTIFACTS_METADATA);
    patterns.put(prefix + "/artifacts.xml.xz", ARTIFACTS_METADATA);
    patterns.put(prefix + "/artifacts.xml", ARTIFACTS_METADATA);
    patterns.put(prefix + "/{dir:features|plugins}/" + NAME_VERSION + ".{" + EXTENSION + ":.*}", BUNDLE);
    patterns.put(prefix + "/{dir:binary}/" + NAME_VERSION, BINARY_BUNDLE);
    return patterns;
  }

  private static String match(final State state, final String name) {
    checkNotNull(state);
    String result = state.getTokens().get(name);
    checkNotNull(result);
    return result;
  }

  public static String name(final State state) {
    return match(state, "name");
  }

  public static String version(final State state) {
    return match(state, "version");
  }

  public static String extension(final State state) {
    return match(state, EXTENSION);
  }

  public static String filename(final State state) {
    return name(state) + '_' + version(state) + '.' + extension(state);
  }

  @Nullable
  public static String site(final State state) {
    return state.getTokens().get(SITE);
  }

  public static State matcherState(final Context context) {
    return context.getAttributes().require(State.class);
  }

  public static P2Attributes toP2Attributes(final String path, final State state) {
    return P2Attributes.builder()
        .componentName(name(state))
        .componentVersion(version(state))
        .extension(extension(state))
        .fileName(filename(state))
        .path(path)
        .build();
  }

  public static P2Attributes toP2AttributesBinary(final String path, final State state) {
    return P2Attributes.builder()
        .pluginName(name(state))
        .componentName(name(state))
        .componentVersion(version(state))
        .path(path)
        .build();
  }
}
