package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.StringWriter;
import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.PublicKey;
import java.security.Security;
import java.util.Arrays;
import java.util.Base64;

import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.openssl.PKCS8Generator;
import org.bouncycastle.openssl.jcajce.JcaPEMWriter;
import org.bouncycastle.openssl.jcajce.JcaPKCS8Generator;
import org.bouncycastle.openssl.jcajce.JceOpenSSLPKCS8EncryptorBuilder;
import org.bouncycastle.openssl.jcajce.JcePEMEncryptorBuilder;

final class TestKeys
{
  final KeyPair keyPair;

  private TestKeys(final KeyPair keyPair) {
    this.keyPair = keyPair;
  }

  static TestKeys generate() throws Exception {
    if (Security.getProvider(BouncyCastleProvider.PROVIDER_NAME) == null) {
      Security.addProvider(new BouncyCastleProvider());
    }
    KeyPairGenerator generator = KeyPairGenerator.getInstance("RSA");
    generator.initialize(2048);
    return new TestKeys(generator.generateKeyPair());
  }

  String pkcs8() {
    return pem("PRIVATE KEY", keyPair.getPrivate().getEncoded());
  }

  String pkcs1() {
    byte[] pkcs8 = keyPair.getPrivate().getEncoded();
    return pem("RSA PRIVATE KEY", Arrays.copyOfRange(pkcs8, 26, pkcs8.length));
  }

  String encryptedPkcs8(final String passphrase) throws Exception {
    JceOpenSSLPKCS8EncryptorBuilder encryptor = new JceOpenSSLPKCS8EncryptorBuilder(PKCS8Generator.AES_256_CBC);
    encryptor.setPassword(passphrase.toCharArray());
    return write(new JcaPKCS8Generator(keyPair.getPrivate(), encryptor.build()));
  }

  String encryptedPkcs1(final String passphrase) throws Exception {
    StringWriter out = new StringWriter();
    try (JcaPEMWriter writer = new JcaPEMWriter(out)) {
      writer.writeObject(keyPair.getPrivate(), new JcePEMEncryptorBuilder("AES-256-CBC").build(passphrase.toCharArray()));
    }
    return out.toString();
  }

  private static String write(final Object object) throws Exception {
    StringWriter out = new StringWriter();
    try (JcaPEMWriter writer = new JcaPEMWriter(out)) {
      writer.writeObject(object);
    }
    return out.toString();
  }

  PublicKey publicKey() {
    return keyPair.getPublic();
  }

  String publicKeyBase64() {
    return Base64.getEncoder().encodeToString(keyPair.getPublic().getEncoded());
  }

  private static String pem(final String type, final byte[] der) {
    return "-----BEGIN " + type + "-----\n"
        + Base64.getMimeEncoder(64, new byte[]{'\n'}).encodeToString(der)
        + "\n-----END " + type + "-----\n";
  }
}
