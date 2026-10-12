package org.globalart.astrum.repository.swift.internal;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.swift.AssetKind;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public final class SwiftPaths
{
  public static final String IDENTIFIERS = "/identifiers";

  public static final String LOGIN = "/login";

  public static final String AVAILABILITY = "/availability";

  public static final String MANIFEST_FILENAME = Router.LOCAL_ATTRIBUTE_PREFIX + "swift.manifest.filename";

  public static final String UPSTREAM_SWIFT_VERSION = Router.LOCAL_ATTRIBUTE_PREFIX + "swift.upstream.swift-version";

  public static final String SWIFT_VERSION_PARAMETER = "swift-version";

  private static final Pattern RELEASES = Pattern.compile("/([^/]+)/([^/.]+)(\\.json)?");

  private static final Pattern RELEASE = Pattern.compile("/([^/]+)/([^/]+)/([^/]+?)(\\.json)?");

  private static final Pattern ARCHIVE = Pattern.compile("/([^/]+)/([^/]+)/([^/]+)\\.zip");

  private static final Pattern MANIFEST = Pattern.compile("/([^/]+)/([^/]+)/([^/]+)/([^/]+)");

  private SwiftPaths() {
  }

  public static org.globalart.astrum.repository.view.Matcher releases() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      Matcher matcher = RELEASES.matcher(context.getRequest().getPath());
      return matcher.matches() && set(context, matcher.group(1), matcher.group(2), null, AssetKind.RELEASES);
    });
  }

  public static org.globalart.astrum.repository.view.Matcher release() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), releaseMatcher());
  }

  public static org.globalart.astrum.repository.view.Matcher publish() {
    return LogicMatchers.and(new ActionMatcher(PUT), context -> {
      Matcher matcher = RELEASE.matcher(context.getRequest().getPath());
      return matcher.matches() && matcher.group(4) == null
          && set(context, matcher.group(1), matcher.group(2), matcher.group(3), AssetKind.ARCHIVE);
    });
  }

  public static org.globalart.astrum.repository.view.Matcher archive() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      Matcher matcher = ARCHIVE.matcher(context.getRequest().getPath());
      return matcher.matches() && set(context, matcher.group(1), matcher.group(2), matcher.group(3), AssetKind.ARCHIVE);
    });
  }

  public static org.globalart.astrum.repository.view.Matcher manifest() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      Matcher matcher = MANIFEST.matcher(context.getRequest().getPath());
      if (!matcher.matches() || !Manifests.isManifest(matcher.group(4))) {
        return false;
      }
      context.getAttributes().set(MANIFEST_FILENAME, matcher.group(4));
      return set(context, matcher.group(1), matcher.group(2), matcher.group(3), AssetKind.MANIFEST);
    });
  }

  public static org.globalart.astrum.repository.view.Matcher identifiers() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(IDENTIFIERS));
  }

  public static org.globalart.astrum.repository.view.Matcher availability() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(AVAILABILITY));
  }

  public static org.globalart.astrum.repository.view.Matcher login() {
    return LogicMatchers.and(new ActionMatcher(POST),
        LogicMatchers.or(new LiteralMatcher(LOGIN), new LiteralMatcher("/"), new LiteralMatcher("")));
  }

  private static org.globalart.astrum.repository.view.Matcher releaseMatcher() {
    return context -> {
      Matcher matcher = RELEASE.matcher(context.getRequest().getPath());
      return matcher.matches() && set(context, matcher.group(1), matcher.group(2), matcher.group(3), AssetKind.RELEASE);
    };
  }

  private static boolean set(
      final Context context,
      final String scope,
      final String name,
      @Nullable final String version,
      final AssetKind kind)
  {
    if (!SwiftCoordinates.isValidScope(scope) || !SwiftCoordinates.isValidName(name)
        || (version != null && !SwiftVersion.isValid(version))) {
      return false;
    }
    context.getAttributes().set(SwiftCoordinates.class, new SwiftCoordinates(scope, name, version));
    context.getAttributes().set(AssetKind.class, kind);
    return true;
  }

  public static SwiftCoordinates coordinates(final Context context) {
    return context.getAttributes().require(SwiftCoordinates.class);
  }

  public static AssetKind kind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  public static String manifestFilename(final Context context) {
    String filename = context.getAttributes().get(MANIFEST_FILENAME, String.class);
    return filename == null ? Manifests.MANIFEST : filename;
  }

  @Nullable
  public static String swiftVersion(final Context context) {
    String value = context.getRequest().getParameters().get(SWIFT_VERSION_PARAMETER);
    return value == null || value.isBlank() ? null : value.strip();
  }

  @Nullable
  public static String header(final Context context, final String name) {
    for (String header : context.getRequest().getHeaders().names()) {
      if (header.equalsIgnoreCase(name)) {
        return context.getRequest().getHeaders().get(header);
      }
    }
    return null;
  }

  public static String baseUrl(final Context context) {
    String url = context.getRepository().getUrl();
    return url.endsWith("/") ? url.substring(0, url.length() - 1) : url;
  }
}
