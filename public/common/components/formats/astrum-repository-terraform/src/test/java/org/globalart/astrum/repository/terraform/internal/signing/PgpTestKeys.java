package org.globalart.astrum.repository.terraform.internal.signing;

import java.io.ByteArrayOutputStream;
import java.security.KeyPairGenerator;
import java.security.Security;
import java.util.Date;

import org.bouncycastle.bcpg.ArmoredOutputStream;
import org.bouncycastle.bcpg.HashAlgorithmTags;
import org.bouncycastle.bcpg.PublicKeyAlgorithmTags;
import org.bouncycastle.bcpg.SymmetricKeyAlgorithmTags;
import org.bouncycastle.bcpg.sig.KeyFlags;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.openpgp.PGPKeyPair;
import org.bouncycastle.openpgp.PGPKeyRingGenerator;
import org.bouncycastle.openpgp.PGPSecretKeyRing;
import org.bouncycastle.openpgp.PGPSignature;
import org.bouncycastle.openpgp.PGPSignatureSubpacketGenerator;
import org.bouncycastle.openpgp.operator.PGPDigestCalculator;
import org.bouncycastle.openpgp.operator.jcajce.JcaPGPContentSignerBuilder;
import org.bouncycastle.openpgp.operator.jcajce.JcaPGPDigestCalculatorProviderBuilder;
import org.bouncycastle.openpgp.operator.jcajce.JcaPGPKeyPair;
import org.bouncycastle.openpgp.operator.jcajce.JcePBESecretKeyEncryptorBuilder;

import static java.nio.charset.StandardCharsets.US_ASCII;

/**
 * OpenPGP key rings for tests, as {@code gpg --armor --export-secret-keys} would write them.
 */
public final class PgpTestKeys
{
  private PgpTestKeys() {
  }

  public static void installProvider() {
    if (Security.getProvider(BouncyCastleProvider.PROVIDER_NAME) == null) {
      Security.addProvider(new BouncyCastleProvider());
    }
  }

  /**
   * An armored secret key ring with one RSA key that may certify and sign.
   */
  public static String secretKeyRing(final String userId, final String passphrase) throws Exception {
    installProvider();
    KeyPairGenerator generator = KeyPairGenerator.getInstance("RSA", BouncyCastleProvider.PROVIDER_NAME);
    generator.initialize(2048);
    PGPKeyPair keyPair = new JcaPGPKeyPair(PublicKeyAlgorithmTags.RSA_GENERAL, generator.generateKeyPair(),
        new Date());
    PGPSignatureSubpacketGenerator hashed = new PGPSignatureSubpacketGenerator();
    hashed.setKeyFlags(false, KeyFlags.CERTIFY_OTHER | KeyFlags.SIGN_DATA);
    PGPDigestCalculator sha1 = new JcaPGPDigestCalculatorProviderBuilder().build().get(HashAlgorithmTags.SHA1);
    PGPKeyRingGenerator rings = new PGPKeyRingGenerator(PGPSignature.POSITIVE_CERTIFICATION, keyPair, userId, sha1,
        hashed.generate(), null,
        new JcaPGPContentSignerBuilder(keyPair.getPublicKey().getAlgorithm(), HashAlgorithmTags.SHA256)
            .setProvider(BouncyCastleProvider.PROVIDER_NAME),
        new JcePBESecretKeyEncryptorBuilder(SymmetricKeyAlgorithmTags.AES_256, sha1)
            .setProvider(BouncyCastleProvider.PROVIDER_NAME).build(passphrase.toCharArray()));
    PGPSecretKeyRing ring = rings.generateSecretKeyRing();
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ArmoredOutputStream armored = new ArmoredOutputStream(out)) {
      ring.encode(armored);
    }
    return out.toString(US_ASCII);
  }
}
