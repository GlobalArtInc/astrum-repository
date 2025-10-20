
package org.globalart.goodies.packageurl;

/**
 * Thrown when package-url is detected to be invalid for some reason.
 *
 * @since 1.1.0
 */
public class InvalidException
    extends RuntimeException
{
  InvalidException(final String message) {
    super(message);
  }
}
