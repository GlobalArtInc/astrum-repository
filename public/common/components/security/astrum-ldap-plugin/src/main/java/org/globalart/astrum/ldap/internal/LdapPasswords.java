package org.globalart.astrum.ldap.internal;

import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Arrays;
import java.util.Base64;
import java.util.Locale;

/**
 * Checks a password against the value of a user password attribute: plain text, or one of the RFC 2307 style hashes
 * {@code {SHA}}, {@code {SSHA}}, {@code {SHA256}}, {@code {SSHA256}}, {@code {SHA384}}, {@code {SSHA384}},
 * {@code {SHA512}}, {@code {SSHA512}}, {@code {MD5}} and {@code {SMD5}}.
 */
final class LdapPasswords
{
  private LdapPasswords() {
  }

  static boolean matches(final char[] password, final byte[] stored) {
    if (password == null || password.length == 0 || stored == null || stored.length == 0) {
      return false;
    }
    byte[] candidate = utf8(password);
    try {
      String value = new String(stored, StandardCharsets.UTF_8);
      int end = value.indexOf('}');
      if (!value.startsWith("{") || end < 0) {
        return MessageDigest.isEqual(candidate, stored);
      }
      String scheme = value.substring(1, end).toUpperCase(Locale.ENGLISH);
      String encoded = value.substring(end + 1);
      return switch (scheme) {
        case "CLEARTEXT", "PLAIN" -> MessageDigest.isEqual(candidate, encoded.getBytes(StandardCharsets.UTF_8));
        case "SHA", "SHA1" -> hashed("SHA-1", 20, false, candidate, encoded);
        case "SSHA", "SSHA1" -> hashed("SHA-1", 20, true, candidate, encoded);
        case "SHA256", "SHA-256" -> hashed("SHA-256", 32, false, candidate, encoded);
        case "SSHA256", "SSHA-256" -> hashed("SHA-256", 32, true, candidate, encoded);
        case "SHA384", "SHA-384" -> hashed("SHA-384", 48, false, candidate, encoded);
        case "SSHA384", "SSHA-384" -> hashed("SHA-384", 48, true, candidate, encoded);
        case "SHA512", "SHA-512" -> hashed("SHA-512", 64, false, candidate, encoded);
        case "SSHA512", "SSHA-512" -> hashed("SHA-512", 64, true, candidate, encoded);
        case "MD5" -> hashed("MD5", 16, false, candidate, encoded);
        case "SMD5" -> hashed("MD5", 16, true, candidate, encoded);
        default -> false;
      };
    }
    finally {
      Arrays.fill(candidate, (byte) 0);
    }
  }

  private static boolean hashed(
      final String algorithm,
      final int length,
      final boolean salted,
      final byte[] candidate,
      final String encoded)
  {
    byte[] decoded;
    try {
      decoded = Base64.getDecoder().decode(encoded.trim());
    }
    catch (IllegalArgumentException e) {
      return false;
    }
    if (salted ? decoded.length <= length : decoded.length != length) {
      return false;
    }
    try {
      MessageDigest digest = MessageDigest.getInstance(algorithm);
      digest.update(candidate);
      if (salted) {
        digest.update(decoded, length, decoded.length - length);
      }
      return MessageDigest.isEqual(digest.digest(), Arrays.copyOf(decoded, length));
    }
    catch (NoSuchAlgorithmException e) {
      return false;
    }
  }

  static byte[] utf8(final char[] chars) {
    ByteBuffer buffer = StandardCharsets.UTF_8.encode(CharBuffer.wrap(chars));
    byte[] bytes = Arrays.copyOfRange(buffer.array(), buffer.position(), buffer.limit());
    Arrays.fill(buffer.array(), (byte) 0);
    return bytes;
  }
}
