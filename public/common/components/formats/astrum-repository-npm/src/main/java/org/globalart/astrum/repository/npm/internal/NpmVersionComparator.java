
package org.globalart.astrum.repository.npm.internal;

import java.util.function.BiFunction;

import org.globalart.astrum.common.app.VersionComparator;

import static org.apache.commons.lang3.StringUtils.isEmpty;

/**
 * Utility class for Npm version comparison
 *
 * @since 3.14
 */
public final class NpmVersionComparator
{
  public static final VersionComparator versionComparator = new VersionComparator();

  public static final BiFunction<String, String, String> extractPackageRootVersionUnlessEmpty = (packageRootVersion, packageVersion) ->
      isEmpty(packageRootVersion) ? packageVersion : packageRootVersion;

  public static final BiFunction<String, String, String> extractAlwaysPackageVersion = (packageRootVersion, packageVersion) -> packageVersion;

  public static final BiFunction<String, String, String> extractNewestVersion = (packageRootVersion, packageVersion) ->
      versionComparator.compare(packageVersion, packageRootVersion) > 0
          ? packageVersion : packageRootVersion;

  private NpmVersionComparator() {
    //NOSONAR
  }
}
