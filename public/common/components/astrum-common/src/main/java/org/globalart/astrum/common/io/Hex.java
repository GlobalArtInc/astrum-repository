
package org.globalart.astrum.common.io;

import com.google.common.io.BaseEncoding;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * HEX encoding helpers.
 *
 * @since 3.0
 */
public class Hex
{
  private static final BaseEncoding HEX = BaseEncoding.base16().lowerCase();

  private Hex() {
    // empty
  }

  /**
   * HEX encode bytes to string.
   */
  public static String encode(final byte[] bytes) {
    return HEX.encode(bytes);
  }

  /**
   * HEX decode string to bytes.
   */
  public static byte[] decode(final String encoded) {
    checkNotNull(encoded);
    return HEX.decode(encoded.toLowerCase());
  }
}
