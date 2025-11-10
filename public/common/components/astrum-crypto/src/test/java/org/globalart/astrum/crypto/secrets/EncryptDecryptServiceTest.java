
package org.globalart.astrum.crypto.secrets;

import org.globalart.astrum.crypto.internal.CryptoHelperImpl;
import org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl;
import org.globalart.astrum.crypto.internal.PbeCipherFactoryImpl;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class EncryptDecryptServiceTest
{

  EncryptDecryptService encryptDecryptService =
      new EncryptDecryptService(new PbeCipherFactoryImpl(new CryptoHelperImpl(false),
          new HashingHandlerFactoryImpl(new CryptoHelperImpl(false)), "PBKDF2WithHmacSHA1", null));

  @Test
  public void testEncryptionDecryption() {
    String testString = "test 123 ==== <>?/.,";
    String encoded = encryptDecryptService.encryptAndEncode(testString);
    String decoded = encryptDecryptService.decodeAndDecrypt(encoded);
    assertEquals(testString, decoded);
  }
}
