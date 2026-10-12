package org.globalart.astrum.repository.terraform.internal.signing;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Date;
import java.util.Iterator;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

import org.bouncycastle.bcpg.ArmoredOutputStream;
import org.bouncycastle.bcpg.BCPGOutputStream;
import org.bouncycastle.bcpg.HashAlgorithmTags;
import org.bouncycastle.bcpg.sig.KeyFlags;
import org.bouncycastle.openpgp.PGPException;
import org.bouncycastle.openpgp.PGPPrivateKey;
import org.bouncycastle.openpgp.PGPPublicKey;
import org.bouncycastle.openpgp.PGPPublicKeyRing;
import org.bouncycastle.openpgp.PGPPublicKeyRingCollection;
import org.bouncycastle.openpgp.PGPSecretKey;
import org.bouncycastle.openpgp.PGPSecretKeyRing;
import org.bouncycastle.openpgp.PGPSecretKeyRingCollection;
import org.bouncycastle.openpgp.PGPSignature;
import org.bouncycastle.openpgp.PGPSignatureGenerator;
import org.bouncycastle.openpgp.PGPSignatureList;
import org.bouncycastle.openpgp.PGPSignatureSubpacketGenerator;
import org.bouncycastle.openpgp.PGPSignatureSubpacketVector;
import org.bouncycastle.openpgp.PGPUtil;
import org.bouncycastle.openpgp.jcajce.JcaPGPObjectFactory;
import org.bouncycastle.openpgp.operator.jcajce.JcaKeyFingerprintCalculator;
import org.bouncycastle.openpgp.operator.jcajce.JcaPGPContentSignerBuilder;
import org.bouncycastle.openpgp.operator.jcajce.JcaPGPContentVerifierBuilderProvider;
import org.bouncycastle.openpgp.operator.jcajce.JcePBESecretKeyDecryptorBuilder;

import static java.nio.charset.StandardCharsets.US_ASCII;

/**
 * The OpenPGP operations of provider releases: Terraform verifies a binary detached signature of the
 * {@code SHA256SUMS} document against the ASCII-armored public keys the registry lists.
 */
public final class Pgp
{
  private static final String PROVIDER = "BC";

  private Pgp() {
  }

  /**
   * A private key able to sign, with the public key ring Terraform needs to verify its signatures.
   */
  public record SigningKey(PGPPrivateKey privateKey, PGPPublicKey signingKey, PGPPublicKeyRing publicKeyRing)
  {
    public String keyId() {
      return keyId(publicKeyRing.getPublicKey());
    }

    public String armoredPublicKey() {
      return armor(publicKeyRing);
    }

    static String keyId(final PGPPublicKey key) {
      return String.format(Locale.ROOT, "%016X", key.getKeyID());
    }
  }

  /**
   * An ASCII-armored public key ring and the id of its primary key.
   */
  public record PublicKey(String keyId, String armored)
  {
  }

  /**
   * Reads the first key of an armored or binary secret key ring that may sign data.
   *
   * @throws IllegalArgumentException when the text holds no usable signing key or the passphrase does not open it
   */
  public static SigningKey readSigningKey(final String secretKeyRing, @Nullable final String passphrase) {
    byte[] data = secretKeyRing.strip().getBytes(US_ASCII);
    try (InputStream in = PGPUtil.getDecoderStream(new ByteArrayInputStream(data))) {
      PGPSecretKeyRingCollection rings = new PGPSecretKeyRingCollection(in, new JcaKeyFingerprintCalculator());
      Iterator<PGPSecretKeyRing> iterator = rings.getKeyRings();
      while (iterator.hasNext()) {
        PGPSecretKeyRing ring = iterator.next();
        PGPSecretKey secretKey = signingSecretKey(ring);
        if (secretKey == null) {
          continue;
        }
        char[] pass = passphrase == null ? new char[0] : passphrase.toCharArray();
        PGPPrivateKey privateKey = secretKey.extractPrivateKey(
            new JcePBESecretKeyDecryptorBuilder().setProvider(PROVIDER).build(pass));
        if (privateKey == null) {
          throw new IllegalArgumentException("The signing key has no private part");
        }
        List<PGPPublicKey> publicKeys = new ArrayList<>();
        ring.getPublicKeys().forEachRemaining(publicKeys::add);
        return new SigningKey(privateKey, secretKey.getPublicKey(), new PGPPublicKeyRing(publicKeys));
      }
    }
    catch (PGPException e) {
      throw new IllegalArgumentException("Cannot open the signing key: " + e.getMessage(), e);
    }
    catch (IOException | RuntimeException e) {
      if (e instanceof IllegalArgumentException illegal) {
        throw illegal;
      }
      throw new IllegalArgumentException("Not an OpenPGP secret key: " + e.getMessage(), e);
    }
    throw new IllegalArgumentException("The key ring holds no key that may sign");
  }

  /**
   * A binary detached signature of a document.
   */
  public static byte[] sign(final SigningKey key, final byte[] document) {
    try {
      PGPSignatureGenerator generator = new PGPSignatureGenerator(
          new JcaPGPContentSignerBuilder(key.signingKey().getAlgorithm(), HashAlgorithmTags.SHA256)
              .setProvider(PROVIDER));
      generator.init(PGPSignature.BINARY_DOCUMENT, key.privateKey());
      PGPSignatureSubpacketGenerator hashed = new PGPSignatureSubpacketGenerator();
      hashed.setSignatureCreationTime(false, new Date());
      hashed.setIssuerKeyID(false, key.signingKey().getKeyID());
      generator.setHashedSubpackets(hashed.generate());
      generator.update(document);
      ByteArrayOutputStream out = new ByteArrayOutputStream();
      try (BCPGOutputStream bcpg = new BCPGOutputStream(out)) {
        generator.generate().encode(bcpg);
      }
      return out.toByteArray();
    }
    catch (PGPException | IOException e) {
      throw new IllegalStateException("Cannot sign: " + e.getMessage(), e);
    }
  }

  /**
   * Reads an armored or binary public key ring, as {@code gpg --export} writes it.
   *
   * @throws IllegalArgumentException when the data holds no public key
   */
  public static PublicKey readPublicKey(final byte[] data) {
    try (InputStream in = PGPUtil.getDecoderStream(new ByteArrayInputStream(data))) {
      PGPPublicKeyRingCollection rings = new PGPPublicKeyRingCollection(in, new JcaKeyFingerprintCalculator());
      Iterator<PGPPublicKeyRing> iterator = rings.getKeyRings();
      if (!iterator.hasNext()) {
        throw new IllegalArgumentException("No public key found");
      }
      PGPPublicKeyRing ring = iterator.next();
      return new PublicKey(SigningKey.keyId(ring.getPublicKey()), armor(ring));
    }
    catch (PGPException | IOException | RuntimeException e) {
      if (e instanceof IllegalArgumentException illegal) {
        throw illegal;
      }
      throw new IllegalArgumentException("Not an OpenPGP public key: " + e.getMessage(), e);
    }
  }

  /**
   * The binary form of a detached signature, which may have been uploaded ASCII-armored.
   *
   * @throws IllegalArgumentException when the data is no OpenPGP signature
   */
  public static byte[] binarySignature(final byte[] data) {
    try {
      PGPSignature signature = readSignature(data);
      ByteArrayOutputStream out = new ByteArrayOutputStream();
      try (BCPGOutputStream bcpg = new BCPGOutputStream(out)) {
        signature.encode(bcpg);
      }
      return out.toByteArray();
    }
    catch (IOException e) {
      throw new IllegalArgumentException("Not an OpenPGP signature: " + e.getMessage(), e);
    }
  }

  /**
   * Whether a detached signature of a document was made by a key of an armored public key ring.
   */
  public static boolean verify(final byte[] document, final byte[] signature, final String armoredPublicKey) {
    try {
      PGPSignature parsed = readSignature(signature);
      PGPPublicKeyRingCollection rings;
      try (InputStream in = PGPUtil.getDecoderStream(new ByteArrayInputStream(armoredPublicKey.getBytes(US_ASCII)))) {
        rings = new PGPPublicKeyRingCollection(in, new JcaKeyFingerprintCalculator());
      }
      PGPPublicKey key = rings.getPublicKey(parsed.getKeyID());
      if (key == null) {
        return false;
      }
      parsed.init(new JcaPGPContentVerifierBuilderProvider().setProvider(PROVIDER), key);
      parsed.update(document);
      return parsed.verify();
    }
    catch (PGPException | IOException | IllegalArgumentException e) {
      return false;
    }
  }

  private static PGPSignature readSignature(final byte[] data) throws IOException {
    try (InputStream in = PGPUtil.getDecoderStream(new ByteArrayInputStream(data))) {
      JcaPGPObjectFactory factory = new JcaPGPObjectFactory(in);
      Object object = factory.nextObject();
      if (object instanceof PGPSignatureList list && !list.isEmpty()) {
        return list.get(0);
      }
      throw new IllegalArgumentException("No signature found");
    }
  }

  @Nullable
  private static PGPSecretKey signingSecretKey(final PGPSecretKeyRing ring) {
    PGPSecretKey fallback = null;
    Iterator<PGPSecretKey> keys = ring.getSecretKeys();
    while (keys.hasNext()) {
      PGPSecretKey key = keys.next();
      if (!key.isSigningKey() || key.isPrivateKeyEmpty()) {
        continue;
      }
      Integer flags = keyFlags(key.getPublicKey());
      if (flags != null && (flags & KeyFlags.SIGN_DATA) != 0) {
        return key;
      }
      if (flags == null && key.isMasterKey() && fallback == null) {
        fallback = key;
      }
    }
    return fallback;
  }

  @Nullable
  private static Integer keyFlags(final PGPPublicKey key) {
    Iterator<PGPSignature> signatures = key.getSignatures();
    while (signatures.hasNext()) {
      PGPSignatureSubpacketVector hashed = signatures.next().getHashedSubPackets();
      if (hashed != null && hashed.getKeyFlags() != 0) {
        return hashed.getKeyFlags();
      }
    }
    return null;
  }

  private static String armor(final PGPPublicKeyRing ring) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ArmoredOutputStream armored = new ArmoredOutputStream(out)) {
      ring.encode(armored);
    }
    catch (IOException e) {
      throw new IllegalStateException(e);
    }
    return out.toString(US_ASCII);
  }
}
