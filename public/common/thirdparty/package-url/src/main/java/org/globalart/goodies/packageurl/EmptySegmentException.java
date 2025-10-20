
package org.globalart.goodies.packageurl;

/**
 * Thrown when segmented property has an empty value.
 *
 * @since 1.1.0
 */
public class EmptySegmentException
    extends InvalidException
{
  EmptySegmentException(final String value) {
    super("Empty segment in: " + value);
  }
}
