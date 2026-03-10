
package org.globalart.astrum.supportzip;

/**
 * Allows customization of {@link SupportBundle} for support ZIP file generation.
 *
 * @since 2.7
 */
public interface SupportBundleCustomizer
{
  /**
   * Customize the given bundle, adding one or more content sources.
   */
  void customize(SupportBundle supportBundle);
}
