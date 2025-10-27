
package org.globalart.astrum.common.hash;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import com.google.common.hash.HashCode;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import org.springframework.stereotype.Component;

/**
 * @since 3.35
 * @deprecated please use the appropriate ingest methods on {@code StorageFacet} and {@code FluentBlobs}
 */
@Deprecated
@Component
public class HashAlgorithmHelper
{
  private final int bufferSize;

  @Autowired
  public HashAlgorithmHelper(
      @Value("${astrum.calculateChecksums.bufferSize:32768}") final int bufferSize)
  {
    checkState(bufferSize > 0, String.format("Buffer size should be a positive value: %s", bufferSize));
    this.bufferSize = bufferSize;
  }

  /**
   * Calculate checksums by given hash algorithms
   */
  public Map<HashAlgorithm, HashCode> calculateChecksums(
      final File content,
      final Iterable<HashAlgorithm> hashAlgorithms) throws IOException
  {
    checkNotNull(content);
    checkNotNull(hashAlgorithms);

    Map<HashAlgorithm, MessageDigest> hashAlgorithmMessageDigestMap = new HashMap<>();
    for (HashAlgorithm hashAlgorithm : hashAlgorithms) {
      MessageDigest messageDigest = getInstance(hashAlgorithm.name());
      hashAlgorithmMessageDigestMap.put(hashAlgorithm, messageDigest);
    }

    return calculateChecksumsBuffered(content, hashAlgorithmMessageDigestMap, bufferSize);
  }

  private Map<HashAlgorithm, HashCode> calculateChecksumsBuffered(
      final File content,
      final Map<HashAlgorithm, MessageDigest> hashAlgorithmMessageDigestMap,
      final int bufferSize) throws IOException
  {
    try (InputStream inputStream = new FileInputStream(content)) {
      int n = 0;
      byte[] buffer = new byte[bufferSize];
      while (n != -1) {
        n = inputStream.read(buffer);
        if (n > 0) {
          for (MessageDigest messageDigest : hashAlgorithmMessageDigestMap.values()) {
            messageDigest.update(buffer, 0, n);
          }
        }
      }
    }

    return hashAlgorithmMessageDigestMap
        .entrySet()
        .stream()
        .collect(Collectors.toMap(
            Entry::getKey,
            x -> HashCode.fromBytes(x.getValue().digest()),
            (x, y) -> x, LinkedHashMap::new));
  }

  private MessageDigest getInstance(final String algorithm) {
    try {
      return MessageDigest.getInstance(algorithm);
    }
    catch (NoSuchAlgorithmException e) {
      throw new RuntimeException(e);
    }
  }
}
