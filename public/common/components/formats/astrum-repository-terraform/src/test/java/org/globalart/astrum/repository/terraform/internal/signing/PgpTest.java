package org.globalart.astrum.repository.terraform.internal.signing;

import org.junit.BeforeClass;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.US_ASCII;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

public class PgpTest
{
  private static String secretKey;

  @BeforeClass
  public static void generateKey() throws Exception {
    secretKey = PgpTestKeys.secretKeyRing("Astrum Test <test@example.com>", "secret");
  }

  @Test
  public void signsChecksumsSoThatThePublicKeyVerifiesThem() {
    Pgp.SigningKey key = Pgp.readSigningKey(secretKey, "secret");
    byte[] sums = "abc  terraform-provider-demo_1.0.0_linux_amd64.zip\n".getBytes(UTF_8);

    byte[] signature = Pgp.sign(key, sums);

    assertThat(signature[0] & 0x80).as("binary packet, not armored").isEqualTo(0x80);
    assertThat(Pgp.verify(sums, signature, key.armoredPublicKey())).isTrue();
    assertThat(Pgp.verify("other".getBytes(UTF_8), signature, key.armoredPublicKey())).isFalse();
    assertThat(key.keyId()).matches("[0-9A-F]{16}");
    assertThat(key.armoredPublicKey()).startsWith("-----BEGIN PGP PUBLIC KEY BLOCK-----");
  }

  @Test
  public void rejectsAWrongPassphrase() {
    assertThatThrownBy(() -> Pgp.readSigningKey(secretKey, "wrong"))
        .isInstanceOf(IllegalArgumentException.class);
  }

  @Test
  public void rejectsTextThatIsNoKey() {
    assertThatThrownBy(() -> Pgp.readSigningKey("not a key", null)).isInstanceOf(IllegalArgumentException.class);
    assertThatThrownBy(() -> Pgp.readPublicKey("not a key".getBytes(US_ASCII)))
        .isInstanceOf(IllegalArgumentException.class);
  }

  @Test
  public void readsAPublicKeyAndKeepsItsId() {
    Pgp.SigningKey key = Pgp.readSigningKey(secretKey, "secret");

    Pgp.PublicKey publicKey = Pgp.readPublicKey(key.armoredPublicKey().getBytes(US_ASCII));

    assertThat(publicKey.keyId()).isEqualTo(key.keyId());
    assertThat(publicKey.armored()).contains("PGP PUBLIC KEY BLOCK");
  }

  @Test
  public void turnsAnArmoredSignatureIntoTheBinaryOneTerraformReads() throws Exception {
    Pgp.SigningKey key = Pgp.readSigningKey(secretKey, "secret");
    byte[] document = "document".getBytes(UTF_8);
    byte[] binary = Pgp.sign(key, document);
    java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
    try (org.bouncycastle.bcpg.ArmoredOutputStream armored = new org.bouncycastle.bcpg.ArmoredOutputStream(out)) {
      armored.write(binary);
    }

    byte[] converted = Pgp.binarySignature(out.toByteArray());

    assertThat(converted).isEqualTo(binary);
    assertThatThrownBy(() -> Pgp.binarySignature("plain text".getBytes(UTF_8)))
        .isInstanceOf(IllegalArgumentException.class);
  }
}
