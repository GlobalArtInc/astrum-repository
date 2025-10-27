
package org.globalart.astrum.common.i18n;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A message source which prefixes message codes.
 *
 */
public class PrefixingMessageSource
    implements MessageSource
{
  private final MessageSource messages;

  private final String prefix;

  public PrefixingMessageSource(final MessageSource messages, final String prefix) {
    this.messages = checkNotNull(messages);
    this.prefix = checkNotNull(prefix);
  }

  protected String createCode(final String code) {
    checkNotNull(code);
    return prefix + code;
  }

  @Override
  public String getMessage(final String code) {
    return messages.getMessage(createCode(code));
  }

  @Override
  public String getMessage(final String code, final String defaultValue) {
    return messages.getMessage(createCode(code), defaultValue);
  }

  @Override
  public String format(final String code, final Object... args) {
    return messages.format(createCode(code), args);
  }
}
