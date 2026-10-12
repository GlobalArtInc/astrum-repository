package org.globalart.astrum.repository.gitlfs.internal;

import java.util.Optional;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

/**
 * The endpoints of a Git LFS repository.
 * <p>
 * Clients reach the API at {@code <repository>/info/lfs}, the location git-lfs derives from a remote URL, or at the
 * repository root when {@code lfs.url} names the repository itself. Objects stored in the repository can also be read
 * at their asset path, which is what the browse tree links to.
 */
public final class GitLfsPaths
{
  public static final String API_PREFIX = "/info/lfs";

  public static final String BATCH = "/objects/batch";

  public static final String VERIFY = "/objects/verify";

  public static final String LOCKS = "/locks";

  public static final String LOCKS_VERIFY = "/locks/verify";

  private static final Pattern OBJECT = Pattern.compile("/objects/([0-9a-f]{64})");

  private static final Pattern UNLOCK = Pattern.compile("/locks/([^/]+)/unlock");

  public enum Endpoint
  {
    BATCH,
    VERIFY,
    OBJECT,
    ASSET,
    LOCKS,
    LOCKS_VERIFY,
    UNLOCK
  }

  /**
   * A request path resolved to an endpoint.
   *
   * @param endpoint the endpoint
   * @param apiBase  the path the API was reached at, {@link #API_PREFIX} or the empty string for the repository root
   * @param value    the object id of {@link Endpoint#OBJECT} and {@link Endpoint#ASSET}, the lock id of
   *                 {@link Endpoint#UNLOCK}, otherwise {@code null}
   */
  public record Match(Endpoint endpoint, String apiBase, @Nullable String value)
  {
  }

  private GitLfsPaths() {
  }

  public static Optional<Match> match(@Nullable final String path) {
    if (path == null) {
      return Optional.empty();
    }
    String oid = GitLfsObjects.oidFromAssetPath(path);
    if (oid != null) {
      return Optional.of(new Match(Endpoint.ASSET, "", oid));
    }
    String base = path.startsWith(API_PREFIX + "/") ? API_PREFIX : "";
    String relative = path.substring(base.length());
    switch (relative) {
      case BATCH:
        return Optional.of(new Match(Endpoint.BATCH, base, null));
      case VERIFY:
        return Optional.of(new Match(Endpoint.VERIFY, base, null));
      case LOCKS:
        return Optional.of(new Match(Endpoint.LOCKS, base, null));
      case LOCKS_VERIFY:
        return Optional.of(new Match(Endpoint.LOCKS_VERIFY, base, null));
      default:
        break;
    }
    var object = OBJECT.matcher(relative);
    if (object.matches()) {
      return Optional.of(new Match(Endpoint.OBJECT, base, object.group(1)));
    }
    var unlock = UNLOCK.matcher(relative);
    if (unlock.matches()) {
      return Optional.of(new Match(Endpoint.UNLOCK, base, unlock.group(1)));
    }
    return Optional.empty();
  }

  public static Matcher batch() {
    return endpoint(Endpoint.BATCH, POST);
  }

  public static Matcher verify() {
    return endpoint(Endpoint.VERIFY, POST);
  }

  public static Matcher upload() {
    return endpoint(Endpoint.OBJECT, PUT);
  }

  public static Matcher download() {
    return LogicMatchers.or(endpoint(Endpoint.OBJECT, GET, HEAD), endpoint(Endpoint.ASSET, GET, HEAD));
  }

  public static Matcher locks() {
    return endpoint(Endpoint.LOCKS, GET, HEAD, POST);
  }

  public static Matcher locksVerify() {
    return endpoint(Endpoint.LOCKS_VERIFY, POST);
  }

  public static Matcher unlock() {
    return endpoint(Endpoint.UNLOCK, POST);
  }

  /**
   * @return the endpoint the matching route resolved
   */
  public static Match matched(final Context context) {
    return context.getAttributes().require(Match.class);
  }

  /**
   * @return the absolute URL of the API as the client reached it
   */
  public static String apiUrl(final Context context) {
    return context.getRepository().getUrl() + matched(context).apiBase();
  }

  private static Matcher endpoint(
      final Endpoint endpoint,
      final String... methods)
  {
    return LogicMatchers.and(new ActionMatcher(methods), context -> {
      Optional<Match> match = match(context.getRequest().getPath()).filter(found -> found.endpoint() == endpoint);
      match.ifPresent(found -> context.getAttributes().set(Match.class, found));
      return match.isPresent();
    });
  }
}
