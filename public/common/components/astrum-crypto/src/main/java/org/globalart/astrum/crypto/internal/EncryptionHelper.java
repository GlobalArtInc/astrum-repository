
package org.globalart.astrum.crypto.internal;

import com.fasterxml.jackson.core.Base64Variants;

public final class EncryptionHelper
{
  private EncryptionHelper() {
    // empty
  }

  public static final String KEY_ITERATION_PHC = "key_iteration";

  public static final String KEY_LEN_PHC = "key_len";

  public static final String IV = "iv";

  public static String toBase64(final byte[] value) {
    return Base64Variants.getDefaultVariant().encode(value);
  }

  public static byte[] fromBase64(final String encoded) {
    return Base64Variants.getDefaultVariant().decode(encoded);
  }
}
