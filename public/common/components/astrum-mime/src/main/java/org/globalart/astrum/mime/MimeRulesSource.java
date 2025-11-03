
package org.globalart.astrum.mime;

import javax.annotation.Nullable;

/**
 * Interface to provide hints what a given source expects as MIME type based on resource name.
 *
 * @since 2.0
 */
public interface MimeRulesSource
{
  MimeRulesSource NOOP = new MimeRulesSource()
  {
    @Nullable
    @Override
    public MimeRule getRuleForName(final String name) {
      return null;
    }
  };

  /**
   * Returns the forced MIME type that corresponds (should correspond) to given name in the context of given rule
   * source. Returns {@code null} if no rules found.
   */
  @Nullable
  MimeRule getRuleForName(String name);
}
