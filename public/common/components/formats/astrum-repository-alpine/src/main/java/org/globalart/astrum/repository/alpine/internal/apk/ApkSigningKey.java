package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.security.RsaUtils;

import static com.google.common.base.Preconditions.checkNotNull;

public final class ApkSigningKey
{
  private final String keyName;

  private final String privateKeyPem;

  @Nullable
  private final String passphrase;

  private final String publicKeyPem;

  private final String fingerprint;

  private ApkSigningKey(
      final String keyName,
      final String privateKeyPem,
      @Nullable final String passphrase,
      final String publicKeyPem)
  {
    this.keyName = checkNotNull(keyName);
    this.privateKeyPem = privateKeyPem;
    this.passphrase = passphrase;
    this.publicKeyPem = publicKeyPem;
    this.fingerprint = HexFormat.of().formatHex(sha256(publicKeyPem.replaceAll("\\s", "")));
  }

  public static ApkSigningKey parse(final String keyName, final String privateKeyPem, @Nullable final String passphrase) {
    try {
      byte[] publicKey = RsaUtils.getPublicKey(privateKeyPem, passphrase);
      return new ApkSigningKey(keyName, privateKeyPem, passphrase, new String(publicKey, StandardCharsets.US_ASCII));
    }
    catch (IOException | RuntimeException e) {
      String reason = e.getCause() != null ? e.getCause().getMessage() : null;
      throw new IllegalArgumentException("Unable to read the RSA signing key, check the key and passphrase: "
          + e.getMessage() + (reason != null ? " (" + reason + ")" : ""), e);
    }
  }

  public String keyFileName() {
    return keyName + ".rsa.pub";
  }

  public String signatureEntryName() {
    return ".SIGN.RSA." + keyFileName();
  }

  public String id() {
    return keyFileName() + ":" + fingerprint;
  }

  public String publicKeyPem() {
    return publicKeyPem;
  }

  public byte[] sign(final byte[] data) {
    try {
      return RsaUtils.sign(data, privateKeyPem, passphrase);
    }
    catch (IOException e) {
      throw new UncheckedIOException("Unable to sign the index", e);
    }
  }

  private static byte[] sha256(final String value) {
    try {
      return MessageDigest.getInstance("SHA-256").digest(value.getBytes(StandardCharsets.US_ASCII));
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }
}
