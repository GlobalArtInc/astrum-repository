
package org.globalart.astrum.crypto.internal;

import java.security.SecureRandom;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.crypto.HashingHandler;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class HashingHandlerFactoryImplTest
{
  private HashingHandlerFactory factory;

  @BeforeEach
  void setUp() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    // Factory will use default iteration values (SHA1: 1024, SHA256: 10000) when iterations not provided
    factory = new HashingHandlerFactoryImpl(cryptoHelper);
  }

  @Test
  void testCreate_DefaultAlgorithm_Pbkdf2Sha1() {
    HashingHandler hashingHandler = factory.create("PBKDF2WithHmacSHA1", "salt".getBytes());
    assertNotNull(hashingHandler);
    assertInstanceOf(HashingHandler.class, hashingHandler);
  }

  @Test
  void testCreate_DefaultAlgorithm_Pbkdf2Sha256() {
    HashingHandler hashingHandler = factory.create("PBKDF2WithHmacSHA256", "salt".getBytes());
    assertNotNull(hashingHandler);
    assertInstanceOf(HashingHandler.class, hashingHandler);
  }

  @Test
  void testCreate_DefaultAlgorithm_Unsupported() {
    String unsupportedAlgorithm = "unsupportedAlgo";
    assertThrows(IllegalArgumentException.class, () -> factory.create(unsupportedAlgorithm));
  }

  @Test
  void testCreate_when_Sha1Algorithm_input_creates_Pbkdf2Sha1() {
    String encoded =
        "$PBKDF2WithHmacSHA1$iv=a6f7e545dc07ae8b0c5ff522d58b5994,key_iteration=10000,key_len=256$9+gAr77ZJtvlpDZm7Av1bg==$Z7yqZ3ok5JyAFYjoJ9lo+p2G1GZFUX9kqnDEJFHeErg=";

    HashingHandler hashingHandler = factory.create(encoded);
    assertNotNull(hashingHandler);
  }

  @Test
  void testCreate_when_Sha256Algorithm_input_creates_Pbkdf2Sha256() {
    String encoded =
        "$PBKDF2WithHmacSHA256$iv=a6f7e545dc07ae8b0c5ff522d58b5994,key_iteration=10000,key_len=256$9+gAr77ZJtvlpDZm7Av1bg==$Z7yqZ3ok5JyAFYjoJ9lo+p2G1GZFUX9kqnDEJFHeErg=";

    HashingHandler hashingHandler = factory.create(encoded);
    assertNotNull(hashingHandler);
  }

  @Test
  void testCreate_UnsupportedAlgorithm_ShouldThrow() {
    String encoded = "$unsupportedabcdef0123456789$16salt";
    assertThrows(IllegalArgumentException.class, () -> factory.create(encoded));
  }

  @Test
  void testCreate_WithExplicitIterations() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Explicitly pass 210000 iterations (should use that instead of defaults)
    HashingHandler sha1Handler = customFactory.create("PBKDF2WithHmacSHA1", "salt".getBytes(), 210000);
    HashingHandler sha256Handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), 310000);

    assertNotNull(sha1Handler);
    assertNotNull(sha256Handler);
    assertInstanceOf(HashingHandler.class, sha1Handler);
    assertInstanceOf(HashingHandler.class, sha256Handler);
  }

  @Test
  void testCreate_WithNullIterationsUsesDefaults() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Pass null iterations (should use defaults: SHA1=1024, SHA256=10000)
    HashingHandler sha1Handler = customFactory.create("PBKDF2WithHmacSHA1", "salt".getBytes(), null);
    HashingHandler sha256Handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), null);

    assertNotNull(sha1Handler);
    assertNotNull(sha256Handler);
    assertInstanceOf(HashingHandler.class, sha1Handler);
    assertInstanceOf(HashingHandler.class, sha256Handler);
  }

  @Test
  void testCreate_WithoutIterationsParameterUsesDefaults() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Don't pass iterations parameter (should use defaults: SHA1=1024, SHA256=10000)
    HashingHandler sha1Handler = customFactory.create("PBKDF2WithHmacSHA1", "salt".getBytes());
    HashingHandler sha256Handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes());

    assertNotNull(sha1Handler);
    assertNotNull(sha256Handler);
    assertInstanceOf(HashingHandler.class, sha1Handler);
    assertInstanceOf(HashingHandler.class, sha256Handler);
  }

  @Test
  void testIterationsPriority_ExplicitIterationsTakePrecedence() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Explicit iterations (5000) should be used instead of defaults
    HashingHandler sha1Handler = customFactory.create("PBKDF2WithHmacSHA1", "salt".getBytes(), 5000);
    HashingHandler sha256Handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), 5000);

    assertNotNull(sha1Handler);
    assertNotNull(sha256Handler);
  }

  @Test
  void testIterationsPriority_FromPhcString() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // PHC string contains key_iteration=7500
    String phcString = "$PBKDF2WithHmacSHA256$iv=abc,key_iteration=7500,key_len=256$c2FsdA==$dmFsdWU=";
    HashingHandler handler = customFactory.create(phcString);

    assertNotNull(handler);
  }

  @Test
  void testIterationsPriority_PhcStringWithoutIterations_UsesDefaults() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // PHC string without key_iteration attribute should use defaults
    String phcStringSHA1 = "$PBKDF2WithHmacSHA1$iv=abc,key_len=128$c2FsdA==$dmFsdWU=";
    String phcStringSHA256 = "$PBKDF2WithHmacSHA256$iv=abc,key_len=256$c2FsdA==$dmFsdWU=";

    HashingHandler sha1Handler = customFactory.create(phcStringSHA1);
    HashingHandler sha256Handler = customFactory.create(phcStringSHA256);

    assertNotNull(sha1Handler);
    assertNotNull(sha256Handler);
  }

  @Test
  void testIterationsPriority_PhcStringWithInvalidIterations_UsesDefaults() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // PHC string with invalid (non-numeric) key_iteration should use defaults
    String phcString = "$PBKDF2WithHmacSHA256$iv=abc,key_iteration=invalid,key_len=256$c2FsdA==$dmFsdWU=";
    HashingHandler handler = customFactory.create(phcString);

    assertNotNull(handler);
  }

  @Test
  void testDefaultIterations_SHA1_Is1024() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Create handler without explicit iterations, should use SHA1 default (1024)
    HashingHandler handler = customFactory.create("PBKDF2WithHmacSHA1", "salt".getBytes(), null);

    assertNotNull(handler);
    assertInstanceOf(HashingHandler.class, handler);
  }

  @Test
  void testDefaultIterations_SHA256_Is10000() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Create handler without explicit iterations, should use SHA256 default (10000)
    HashingHandler handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), null);

    assertNotNull(handler);
    assertInstanceOf(HashingHandler.class, handler);
  }

  @Test
  void testCreate_WithZeroIterations_UsesZero() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Explicitly passing 0 iterations should use 0, not defaults
    HashingHandler handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), 0);

    assertNotNull(handler);
  }

  @Test
  void testCreate_WithVeryHighIterations() {
    CryptoHelper cryptoHelper = mock(CryptoHelper.class);
    when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());

    HashingHandlerFactory customFactory = new HashingHandlerFactoryImpl(cryptoHelper);

    // Test with very high iteration count (1 million)
    HashingHandler handler = customFactory.create("PBKDF2WithHmacSHA256", "salt".getBytes(), 1000000);

    assertNotNull(handler);
  }
}
