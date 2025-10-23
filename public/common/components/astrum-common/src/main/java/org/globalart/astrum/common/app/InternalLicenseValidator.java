
package org.globalart.astrum.common.app;

/**
 * Provide information about the internal application license.
 *
 * Exposed for informative use only, replacing this component does not actually change how licensing is applied.
 */
public interface InternalLicenseValidator
{
  /**
   * Returns {@code true} if community license is required for operation.
   */
  boolean isRequired();
}
