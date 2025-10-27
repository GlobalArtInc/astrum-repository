
package org.globalart.astrum.common.hash;

import java.io.FilterInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;
import java.util.function.Consumer;

import javax.annotation.Nonnull;

import com.google.common.hash.HashCode;
import com.google.common.hash.Hasher;
import com.google.common.hash.HashingInputStream;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An {@link InputStream} that maintains multiple hashes and the number of bytes of data read from it.
 *
 * @see HashingInputStream
 * @since 3.0
 */
public class MultiHashingInputStream
    extends FilterInputStream
{
  protected final Map<HashAlgorithm, Hasher> hashers = new LinkedHashMap<>();

  private long count;

  public MultiHashingInputStream(final Iterable<HashAlgorithm> algorithms, final InputStream inputStream) {
    super(checkNotNull(inputStream));
    checkNotNull(algorithms);
    for (HashAlgorithm algorithm : algorithms) {
      hashers.put(algorithm, algorithm.function().newHasher());
    }
  }

  @Override
  public int read() throws IOException {
    waitForHashes();

    int b = in.read();
    if (b != -1) {
      submitHashing(hasher -> hasher.putByte((byte) b));
      count++;
    }
    return b;
  }

  @Override
  public int read(@Nonnull final byte[] bytes, final int off, final int len) throws IOException {
    waitForHashes();

    int numRead = in.read(bytes, off, len);
    if (numRead != -1) {
      // Create a copy of the read bytes in case the provided buffer is externally modified
      byte[] copy = new byte[numRead];
      System.arraycopy(bytes, off, copy, 0, numRead);

      submitHashing(hasher -> hasher.putBytes(copy, 0, numRead));
      count += numRead;
    }
    return numRead;
  }

  @Override
  public boolean markSupported() {
    return false;
  }

  @Override
  public void mark(final int readlimit) {
    // no-op
  }

  @Override
  public void reset() throws IOException {
    throw new IOException("reset not supported");
  }

  /**
   * Gets the {@link HashCode}s based on the data read from this stream.
   */
  public Map<HashAlgorithm, HashCode> hashes() {
    try {
      waitForHashes();
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }

    Map<HashAlgorithm, HashCode> hashes = new HashMap<>(hashers.size());
    for (Entry<HashAlgorithm, Hasher> entry : hashers.entrySet()) {
      hashes.put(entry.getKey(), entry.getValue().hash());
    }
    return hashes;
  }

  /**
   * Gets the number of bytes read from this stream.
   */
  public long count() {
    return count;
  }

  protected void submitHashing(final Consumer<Hasher> runnable) {
    hashers.values().forEach(runnable::accept);
  }

  protected void waitForHashes() throws IOException {
    // not required in this implementation
  }
}
