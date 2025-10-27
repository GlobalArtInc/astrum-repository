
package org.globalart.astrum.common.hash;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Map;

import com.google.common.hash.Funnels;
import com.google.common.hash.HashCode;
import com.google.common.hash.HashFunction;
import com.google.common.hash.Hasher;
import com.google.common.hash.HashingInputStream;
import com.google.common.io.ByteStreams;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link HashCode} helpers.
 *
 * @since 3.0
 */
public final class Hashes
{
  private Hashes() {
    // no instance
  }

  /**
   * Computes the hash of the given stream using the given algorithm.
   */
  public static HashCode hash(final HashAlgorithm algorithm, final InputStream inputStream) throws IOException {
    checkNotNull(algorithm);
    checkNotNull(inputStream);

    try (HashingInputStream hashingStream = new HashingInputStream(algorithm.function(), inputStream)) {
      ByteStreams.copy(hashingStream, ByteStreams.nullOutputStream());
      return hashingStream.hash();
    }
  }

  /**
   * Computes the hash of the given stream using multiple algorithms in one pass.
   */
  public static Map<HashAlgorithm, HashCode> hash(
      final Iterable<HashAlgorithm> algorithms,
      final InputStream inputStream) throws IOException
  {
    checkNotNull(algorithms);
    checkNotNull(inputStream);

    try (MultiHashingInputStream hashingStream = MultiHashingInputStreamFactory.input(algorithms, inputStream)) {
      ByteStreams.copy(hashingStream, ByteStreams.nullOutputStream());
      return hashingStream.hashes();
    }
  }

  /**
   * Computes the hash of the given stream using the given function.
   */
  public static HashCode hash(final HashFunction function, final InputStream input) throws IOException {
    Hasher hasher = function.newHasher();
    OutputStream output = Funnels.asOutputStream(hasher);
    ByteStreams.copy(input, output);
    return hasher.hash();
  }
}
