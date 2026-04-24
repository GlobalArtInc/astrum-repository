package org.globalart.astrum.repository.alpine.internal.apk;

import java.nio.charset.StandardCharsets;
import java.security.Signature;

import org.junit.BeforeClass;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

public class ApkSigningKeyTest
{
  private static TestKeys keys;

  @BeforeClass
  public static void generateKeys() throws Exception {
    keys = TestKeys.generate();
  }

  @Test
  public void readsPkcs8AndPkcs1Keys() {
    for (String pem : new String[]{keys.pkcs8(), keys.pkcs1()}) {
      ApkSigningKey key = ApkSigningKey.parse("repo", pem, null);
      assertEquals(keys.publicKeyBase64(), body(key.publicKeyPem()));
    }
  }

  @Test
  public void readsEncryptedKeys() throws Exception {
    for (String pem : new String[]{keys.encryptedPkcs8("secret"), keys.encryptedPkcs1("secret")}) {
      ApkSigningKey key = ApkSigningKey.parse("repo", pem, "secret");
      assertEquals(keys.publicKeyBase64(), body(key.publicKeyPem()));
    }
  }

  @Test
  public void rejectsMissingOrWrongPassphrase() throws Exception {
    for (String pem : new String[]{keys.encryptedPkcs8("secret"), keys.encryptedPkcs1("secret")}) {
      assertThrows(IllegalArgumentException.class, () -> ApkSigningKey.parse("repo", pem, null));
      assertThrows(IllegalArgumentException.class, () -> ApkSigningKey.parse("repo", pem, "wrong"));
    }
  }

  @Test
  public void rejectsNonKeys() {
    assertThrows(IllegalArgumentException.class, () -> ApkSigningKey.parse("repo", "not a key", null));
    assertThrows(IllegalArgumentException.class, () ->
        ApkSigningKey.parse("repo", "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n", null));
  }

  @Test
  public void namesKeyAfterRepository() {
    ApkSigningKey key = ApkSigningKey.parse("alpine-hosted", keys.pkcs8(), null);
    assertEquals("alpine-hosted.rsa.pub", key.keyFileName());
    assertEquals(".SIGN.RSA.alpine-hosted.rsa.pub", key.signatureEntryName());
    assertEquals(key.id(), ApkSigningKey.parse("alpine-hosted", keys.pkcs1(), null).id());
  }

  @Test
  public void signaturesVerifyWithPublicKey() throws Exception {
    ApkSigningKey key = ApkSigningKey.parse("repo", keys.pkcs1(), null);
    byte[] data = "payload".getBytes(StandardCharsets.UTF_8);
    Signature verifier = Signature.getInstance("SHA1withRSA");
    verifier.initVerify(keys.publicKey());
    verifier.update(data);
    assertTrue(verifier.verify(key.sign(data)));
  }

  static String body(final String pem) {
    return pem.replaceAll("-----[A-Z ]+-----", "").replaceAll("\\s", "");
  }
}
