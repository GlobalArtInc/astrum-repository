
package org.globalart.astrum.common.hash;

import java.util.Map;
import java.util.Optional;

import com.google.common.collect.ImmutableMap;
import com.google.common.hash.HashFunction;
import com.google.common.hash.Hashing;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;

/**
 * A hash algorithm name paired with a {@link HashFunction}.
 *
 * @since 3.0
 */
public class HashAlgorithm
{
  public static final HashAlgorithm MD5 = new HashAlgorithm("md5", Hashing.md5());

  public static final HashAlgorithm SHA1 = new HashAlgorithm("sha1", Hashing.sha1());

  public static final HashAlgorithm SHA256 = new HashAlgorithm("sha256", Hashing.sha256());

  public static final HashAlgorithm SHA512 = new HashAlgorithm("sha512", Hashing.sha512());

  public static final Map<String, HashAlgorithm> ALL_HASH_ALGORITHMS = ImmutableMap
      .of(MD5.name, MD5, SHA1.name, SHA1, SHA256.name, SHA256, SHA512.name, SHA512);

  private final String name;

  private final HashFunction function;

  public HashAlgorithm(String name, HashFunction function) {
    this.name = checkNotNull(name);
    this.function = checkNotNull(function);
  }

  public String name() {
    return name;
  }

  public HashFunction function() {
    return function;
  }

  public static Optional<HashAlgorithm> getHashAlgorithm(final String algorithm) {
    return ofNullable(ALL_HASH_ALGORITHMS.get(algorithm));
  }
}
