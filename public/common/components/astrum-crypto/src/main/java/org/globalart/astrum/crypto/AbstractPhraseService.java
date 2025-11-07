
package org.globalart.astrum.crypto;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Common support for marking and checking customized pass-phrases.
 *
 * @since 3.8
 */
public abstract class AbstractPhraseService
    implements PhraseService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final boolean hasMasterPhrase;

  public AbstractPhraseService(final boolean hasMasterPhrase) {
    this.hasMasterPhrase = hasMasterPhrase;
  }

  protected abstract String getMasterPhrase();

  @Override
  public String getPhrase(final String defaultPhrase) {
    return hasMasterPhrase ? getMasterPhrase() : defaultPhrase;
  }

  @Override
  public String mark(final String value) {
    if (hasMasterPhrase && value != null) {
      if (!value.contains("~{")) {
        return value.replace("{", "~{").replace("}", "}~");
      }
    }
    return value;
  }

  @Override
  public boolean usesLegacyEncoding(final String value) {
    if (hasMasterPhrase && value != null) {
      if (value.contains("~{")) {
        return false;
      }
    }
    return true;
  }
}
