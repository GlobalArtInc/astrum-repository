
package org.globalart.astrum.blobstore;

import java.util.regex.Pattern;

/**
 * Superclass containing common code for {@link LocationStrategy} implementations.
 *
 * @since 3.1
 */
public abstract class LocationStrategySupport
    implements LocationStrategy
{
  private static final Pattern UNSAFE_TOKENS = Pattern.compile("[.\\\\:/]");

  protected String escapeFilename(final String value) {
    return UNSAFE_TOKENS.matcher(value).replaceAll("-");
  }
}
