
package org.globalart.astrum.crypto;

import java.security.KeyPairGenerator;
import java.security.KeyStore;
import java.security.KeyStoreException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.Provider;
import java.security.SecureRandom;
import java.security.Signature;
import java.security.cert.CertificateException;
import java.security.cert.CertificateFactory;

import javax.crypto.Cipher;
import javax.crypto.NoSuchPaddingException;
import javax.crypto.SecretKeyFactory;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.TrustManagerFactory;

/**
 * Provides access to cryptology components.
 *
 * @since 3.0
 */
public interface CryptoHelper
{
  Provider getProvider();

  Cipher createCipher(String transformation) throws NoSuchAlgorithmException, NoSuchPaddingException;

  int getCipherMaxAllowedKeyLength(String transformation) throws NoSuchAlgorithmException;

  Signature createSignature(String algorithm) throws NoSuchAlgorithmException;

  SecureRandom createSecureRandom(String algorithm) throws NoSuchAlgorithmException;

  SecureRandom createSecureRandom();

  KeyStore createKeyStore(String type) throws KeyStoreException;

  KeyPairGenerator createKeyPairGenerator(String algorithm) throws NoSuchAlgorithmException;

  CertificateFactory createCertificateFactory(String type) throws CertificateException;

  KeyManagerFactory createKeyManagerFactory(String algorithm) throws NoSuchAlgorithmException;

  TrustManagerFactory createTrustManagerFactory(String algorithm) throws NoSuchAlgorithmException;

  MessageDigest createDigest(String algorithm) throws NoSuchAlgorithmException;

  SecretKeyFactory createSecretKeyFactory(String algorithm) throws NoSuchAlgorithmException;
}
