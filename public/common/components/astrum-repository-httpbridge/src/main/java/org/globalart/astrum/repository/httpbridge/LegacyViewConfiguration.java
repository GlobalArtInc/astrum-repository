
package org.globalart.astrum.repository.httpbridge;

import java.util.regex.Pattern;

/**
 * Legacy format-specific view configuration.
 *
 * @since 3.7
 */
public interface LegacyViewConfiguration
{
  /*
   * What format is this configuration for?
   */
  String getFormat();

  /*
   * Used to match against the incoming request URL.
   */
  Pattern getRequestPattern();
}
