
package org.globalart.astrum.repository.npm;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.npm.internal.NpmPackageId;

/**
 * Since 3.16
 */
public class NpmCoordinateUtil
{
  private static final Pattern NPM_VERSION_PATTERN = Pattern
      .compile("-(\\d+\\.\\d+\\.\\d+[A-Za-z\\d\\-.+]*)\\.(?:tar\\.gz|tgz)");


  public static String extractVersion(final String npmPath) {
    Matcher matcher = NPM_VERSION_PATTERN.matcher(npmPath);

    return matcher.find() ? matcher.group(1) : "";
  }

  @Nullable
  public static String getPackageIdScope(final String npmPath) {
    return NpmPackageId.parse(npmPath).scope();
  }

  public static String getPackageIdName(final String npmPath) {
    return NpmPackageId.parse(npmPath).name();
  }

  private NpmCoordinateUtil() {
    // no op
  }
}
