
package org.globalart.astrum.repository.search.normalize;

/**
 * Format specific version normalizer
 */
public interface VersionNormalizer
{
  /**
   * Return the version in normalized form for specific format
   */
  String getNormalizedVersion(final String version);
}
