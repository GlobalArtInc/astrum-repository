
package org.globalart.astrum.cleanup.config;

import java.util.Set;

/**
 * Cleanup policy constants.
 *
 * @since 3.24
 */
public interface CleanupPolicyConstants
{
  /**
   * Formats for which the {@code repositories} attachment field is exposed on the
   * cleanup policy XO (both internal and V1 APIs). Gated additionally by the
   * {@code astrum.cleanup.retainAllFormats.enabled} feature flag.
   */
  Set<String> REPOSITORIES_FIELD_SUPPORTED_FORMATS = Set.of(
      "npm", "pypi", "go", "helm", "nuget",
      "yum", "rubygems", "terraform", "swift", "apt", "pub");

  String CLEANUP_ATTRIBUTES_KEY = "cleanup";

  String CLEANUP_NAME_KEY = "policyName";

  String IS_PRERELEASE_KEY = "isPrerelease";

  String LAST_BLOB_UPDATED_KEY = "lastBlobUpdated";

  String LAST_DOWNLOADED_KEY = "lastDownloaded";

  String RETAIN_KEY = "retain";

  String RETAIN_SORT_BY_KEY = "sortBy";

  String REGEX_KEY = "regex";

  String MAVEN2_FORMAT = "maven2";

  String DOCKER_FORMAT = "docker";
}
