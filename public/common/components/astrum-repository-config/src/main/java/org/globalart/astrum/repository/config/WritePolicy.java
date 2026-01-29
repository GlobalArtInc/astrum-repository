
package org.globalart.astrum.repository.config;

/**
 * Write policy.
 *
 * @since 3.0
 */
public enum WritePolicy
{
  /**
   * Asset can be linked with a blob.
   * Asset can be re-linked with another blob.
   * Asset can be unlinked from a blob (blob can be deleted).
   */
  ALLOW,
  /**
   * Asset can be linked with a blob.
   * Asset cannot be re-linked with another blob.
   * Asset can be unlinked from a blob (blob can be deleted).
   */
  ALLOW_ONCE,
  /**
   * Asset cannot be linked with a blob.
   * Asset cannot be re-linked with another blob.
   * Asset cannot be unlinked from a blob.
   */
  DENY;

  /**
   * Returns {@code true} if Create allowed with this policy.
   */
  public boolean checkCreateAllowed() {
    return this == ALLOW || this == ALLOW_ONCE;
  }

  /**
   * Returns {@code true} if Update allowed with this policy.
   */
  public boolean checkUpdateAllowed() {
    return this == ALLOW;
  }

  /**
   * Returns {@code true} if Delete allowed with this policy.
   */
  public boolean checkDeleteAllowed() {
    return this == ALLOW || this == ALLOW_ONCE;
  }

  public static WritePolicy fromString(String value) {
    if (value == null) {
      return null;
    }

    try {
      return WritePolicy.valueOf(value.toUpperCase());
    }
    catch (IllegalArgumentException e) {
      throw new IllegalArgumentException("Invalid writePolicy: '" + value + "'. Valid values: ALLOW, ALLOW_ONCE, DENY",
          e);
    }
  }
}
