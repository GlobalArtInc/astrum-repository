
package org.globalart.astrum.common.i18n;

/**
 * Thrown to indicate a required resource was not found.
 *
 */
public class ResourceNotFoundException
    extends RuntimeException
{
  public ResourceNotFoundException(final String code) {
    super(String.format("Resource not found for code: %s", code));
  }
}
