
package org.globalart.astrum.crypto;

import com.google.common.annotations.VisibleForTesting;

/**
 * Supplies pass-phrases to the various password helpers.
 *
 * @since 3.8
 */
public interface PhraseService
{
  /**
   * Returns currently configured phrase; falls back to the default if none is configured.
   */
  String getPhrase(String defaultPhrase);

  /**
   * Marks the encoded value to make it easier to distinguish it from legacy encoded values.
   */
  String mark(String encoded);

  /**
   * Was the given value encoded using the legacy phrase?
   *
   * @see #mark(String)
   */
  boolean usesLegacyEncoding(String encoded);

  /**
   * Legacy {@link PhraseService} that always returns the default phrase.
   */
  @VisibleForTesting
  PhraseService LEGACY_PHRASE_SERVICE = new AbstractPhraseService(false)
  {
    @Override
    protected String getMasterPhrase() {
      return null;
    }
  };
}
