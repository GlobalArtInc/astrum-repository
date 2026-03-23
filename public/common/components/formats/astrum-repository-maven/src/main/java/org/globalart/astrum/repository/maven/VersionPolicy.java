
package org.globalart.astrum.repository.maven;

/**
 * Repository version policy.
 *
 * @since 3.0
 */
public enum VersionPolicy
{
  /**
   * Only release coordinates allowed.
   */
  RELEASE,

  /**
   * Only snapshot coordinate allowed.
   */
  SNAPSHOT,

  /**
   * Both kind of coordinates allowed.
   */
  MIXED
}
