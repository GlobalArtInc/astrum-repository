package org.globalart.astrum.repository.docker.internal.v2;

import java.util.Set;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

import com.google.common.collect.ImmutableSet;

import static com.google.common.base.Preconditions.checkNotNull;

public class DockerRouteMatcher
    implements Matcher
{
  private static final String NAME = "[a-z0-9]+(?:(?:\\.|_|__|-+)[a-z0-9]+)*(?:/[a-z0-9]+(?:(?:\\.|_|__|-+)[a-z0-9]+)*)*";

  private static final String TAG = "[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}";

  private static final String DIGEST = "[a-z0-9]+(?:[.+_-][a-z0-9]+)*:[a-zA-Z0-9=_-]+";

  private static final String UUID = "[a-zA-Z0-9._=-]+";

  public static final Pattern BASE = Pattern.compile("^/v2/?$");

  public static final Pattern CATALOG = Pattern.compile("^/v2/_catalog$");

  public static final Pattern TAGS = Pattern.compile("^/v2/(" + NAME + ")/tags/list$");

  public static final Pattern MANIFEST = Pattern.compile("^/v2/(" + NAME + ")/manifests/(" + TAG + "|" + DIGEST + ")$");

  public static final Pattern BLOB = Pattern.compile("^/v2/(" + NAME + ")/blobs/(" + DIGEST + ")$");

  public static final Pattern UPLOADS = Pattern.compile("^/v2/(" + NAME + ")/blobs/uploads/?$");

  public static final Pattern UPLOAD = Pattern.compile("^/v2/(" + NAME + ")/blobs/uploads/(" + UUID + ")$");

  public static final Pattern REFERRERS = Pattern.compile("^/v2/(" + NAME + ")/referrers/([^/]+)$");

  private static final Pattern VALID_NAME = Pattern.compile("^" + NAME + "$");

  private static final Pattern VALID_TAG = Pattern.compile("^" + TAG + "$");

  private final DockerRoute.Type type;

  private final Pattern pattern;

  private final Set<String> actions;

  public DockerRouteMatcher(final DockerRoute.Type type, final Pattern pattern, final String... actions) {
    this.type = checkNotNull(type);
    this.pattern = checkNotNull(pattern);
    this.actions = ImmutableSet.copyOf(actions);
  }

  @Override
  public boolean matches(final Context context) {
    if (!actions.isEmpty() && !actions.contains(context.getRequest().getAction())) {
      return false;
    }
    java.util.regex.Matcher matcher = pattern.matcher(context.getRequest().getPath());
    if (!matcher.matches()) {
      return false;
    }
    String name = matcher.groupCount() >= 1 ? matcher.group(1) : null;
    String reference = matcher.groupCount() >= 2 ? matcher.group(2) : null;
    context.getAttributes().set(DockerRoute.class, new DockerRoute(type, name, reference));
    return true;
  }

  public static boolean isValidName(final String name) {
    return name != null && name.length() <= 255 && VALID_NAME.matcher(name).matches();
  }

  public static boolean isValidTag(final String tag) {
    return tag != null && VALID_TAG.matcher(tag).matches();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" + type + " " + pattern + " " + actions + "}";
  }
}
