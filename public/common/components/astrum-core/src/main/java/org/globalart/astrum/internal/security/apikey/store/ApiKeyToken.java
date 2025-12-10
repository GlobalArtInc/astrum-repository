
package org.globalart.astrum.internal.security.apikey.store;

import java.nio.CharBuffer;

import org.globalart.astrum.security.authc.apikey.ApiKey;
import org.globalart.astrum.datastore.mybatis.handlers.PasswordCharacterArrayTypeHandler;

/**
 * {@link ApiKey} token holder; internal use only.
 *
 * This wrapper makes it easy to apply a {@link ApiKeyTokenTypeHandler special} MyBatis type handler to the enclosed
 * character array to allow queries against the encrypted value. The {@link PasswordCharacterArrayTypeHandler default}
 * handler cannot support queries because it adds random salt to every encryption request, so even if the source array
 * was the same the encrypted value would differ each time.
 *
 * @since 3.21
 */
class ApiKeyToken
{
  private final char[] chars;

  public ApiKeyToken(final char[] chars) { // NOSONAR
    this.chars = chars; // NOSONAR: this is just a temporary transfer object
  }

  public char[] getChars() { // NOSONAR
    return chars; // NOSONAR: this is just a temporary transfer object
  }

  public CharBuffer getCharBuffer() {
    return CharBuffer.wrap(chars);
  }
}
