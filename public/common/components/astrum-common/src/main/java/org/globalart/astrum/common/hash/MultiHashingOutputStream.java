
package org.globalart.astrum.common.hash;

import java.io.FilterOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;

import com.google.common.hash.HashCode;
import com.google.common.hash.Hasher;
import com.google.common.hash.HashingOutputStream;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An {@link OutputStream} that maintains multiple hashes.
 *
 * @see HashingOutputStream
 * @since 3.4
 */
public class MultiHashingOutputStream
    extends FilterOutputStream
{
  private final Map<HashAlgorithm, Hasher> hashers = new LinkedHashMap<>();

  public MultiHashingOutputStream(final Iterable<HashAlgorithm> algorithms, final OutputStream out) {
    super(checkNotNull(out));
    checkNotNull(algorithms);
    for (HashAlgorithm algorithm : algorithms) {
      hashers.put(algorithm, algorithm.function().newHasher());
    }
  }

  @Override
  public void write(final int b) throws IOException {
    out.write(b);
    for (Hasher hasher : hashers.values()) {
      hasher.putByte((byte) b);
    }
  }

  /**
   * Gets the {@link HashCode}s based on the data output to this stream.
   *
   * Note this makes a call to Hasher.hash() and therefore can only be called once. Calling this multiple times will
   * result in an exception.
   */
  public Map<HashAlgorithm, HashCode> hashes() {
    Map<HashAlgorithm, HashCode> hashes = new HashMap<>(hashers.size());
    for (Entry<HashAlgorithm, Hasher> entry : hashers.entrySet()) {
      hashes.put(entry.getKey(), entry.getValue().hash());
    }
    return hashes;
  }
}
