package org.globalart.astrum.repository.gitlfs.internal;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

/**
 * Object ids and the asset paths they are stored under.
 * <p>
 * An object id is the lower-case hex SHA-256 of the object content. Objects are stored under
 * {@code /objects/<first two>/<next two>/<oid>}, the layout git-lfs uses in {@code .git/lfs/objects}, so that the
 * browse tree groups objects by the prefix of their id.
 */
public final class GitLfsObjects
{
  public static final String HASH_ALGORITHM = "sha256";

  public static final String OBJECTS_PREFIX = "/objects/";

  private static final Pattern OID = Pattern.compile("[0-9a-f]{64}");

  private static final Pattern ASSET_PATH = Pattern.compile("/objects/([0-9a-f]{2})/([0-9a-f]{2})/([0-9a-f]{64})");

  private GitLfsObjects() {
  }

  public static boolean isValidOid(@Nullable final String oid) {
    return oid != null && OID.matcher(oid).matches();
  }

  public static String assetPath(final String oid) {
    if (!isValidOid(oid)) {
      throw new IllegalArgumentException("Invalid object id: " + oid);
    }
    return OBJECTS_PREFIX + oid.substring(0, 2) + '/' + oid.substring(2, 4) + '/' + oid;
  }

  /**
   * @return the object id stored under the given asset path, or {@code null} when the path is not an object path
   */
  @Nullable
  public static String oidFromAssetPath(@Nullable final String path) {
    if (path == null) {
      return null;
    }
    Matcher matcher = ASSET_PATH.matcher(path);
    if (!matcher.matches()) {
      return null;
    }
    String oid = matcher.group(3);
    return oid.startsWith(matcher.group(1) + matcher.group(2)) ? oid : null;
  }
}
