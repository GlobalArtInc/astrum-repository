
package org.globalart.astrum.repository.content;

/**
 * Different ways content attributes can be updated.
 *
 * @since 3.21
 */
public enum AttributeOperation
{
  /**
   * Sets the attribute under the key, overwriting any existing value.
   */
  SET,

  /**
   * Removes the attribute under the key.
   */
  REMOVE,

  /**
   * Appends a value to the attribute list under the key.
   */
  APPEND,

  /**
   * Prepends a value to the attribute list under the key.
   */
  PREPEND,

  /**
   * Overlays a value onto the attribute map under the key.
   */
  OVERLAY
}
