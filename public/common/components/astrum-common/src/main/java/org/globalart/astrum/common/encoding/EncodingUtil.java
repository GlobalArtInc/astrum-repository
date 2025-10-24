
package org.globalart.astrum.common.encoding;

import java.io.UnsupportedEncodingException;
import java.net.URLDecoder;
import java.net.URLEncoder;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Util class for encoding/decoding strings
 * 
 * @since 3.6
 */
public class EncodingUtil
{
  private EncodingUtil() {
  }

  /**
   * Use a URLEncoder to encode a string
   */
  public static String urlEncode(final String string) {
    try {
      return URLEncoder.encode(string, UTF_8.name());
    }
    catch (UnsupportedEncodingException e) {
      throw new AssertionError("UTF-8 encoding is not supported on this system.", e);
    }
  }

  /**
   * Use a URLDecoder to decode a string
   */
  public static String urlDecode(final String string) {
    try {
      return URLDecoder.decode(string, UTF_8.name());
    }
    catch (UnsupportedEncodingException e) {
      throw new AssertionError("UTF-8 encoding is not supported on this system.", e);
    }
  }

  /**
   * Use a URLDecoder to decode an array of strings and return a new array containing the decoded values
   */
  public static String[] urlDecode(final String... strings) {
    String[] result = null;
    if (strings != null) {
      result = new String[strings.length];
      for (int i = 0; i < strings.length; i++) {
        result[i] = urlDecode(strings[i]);
      }
    }
    return result;
  }
}
