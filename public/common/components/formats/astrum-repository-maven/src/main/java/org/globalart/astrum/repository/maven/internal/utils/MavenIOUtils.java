
package org.globalart.astrum.repository.maven.internal.utils;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.EnumMap;
import java.util.HashMap;
import java.util.Map;
import java.util.Map.Entry;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.maven.MavenPath.HashType;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.PathPayload;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import com.google.common.hash.HashCode;
import com.google.common.hash.HashingOutputStream;

import static org.globalart.astrum.repository.maven.internal.Constants.CHECKSUM_CONTENT_TYPE;

/**
 * The contents of this class is existing code refactored from
 * {@link org.globalart.astrum.repository.maven.internal.orient.MavenFacetUtils}.
 *
 * Specifically, this was done so that both the Orient and SQL database code paths can use the none database specific
 * code now contained herein.
 *
 * The integration tests for Maven will exercise the code in this utility class when running against an Orient or SQL
 * database.
 *
 * @since 3.25
 */
public final class MavenIOUtils
{
  private MavenIOUtils() {
    // no-op
  }

  /**
   * Wrapper to pass in into {@link #createStreamPayload(Path, String, Writer)} to write out actual content.
   */
  public interface Writer
  {
    void write(OutputStream outputStream) throws IOException;
  }

  public static HashedPayload createStreamPayload(
      final Path path,
      final String contentType,
      final Writer writer) throws IOException
  {
    Map<HashAlgorithm, HashingOutputStream> hashingStreams = writeToPath(path, writer);
    Map<HashAlgorithm, HashCode> hashCodes = generateHashCodes(hashingStreams);
    return new HashedPayload(asPayload(path, contentType), hashCodes);
  }

  private static Map<HashAlgorithm, HashingOutputStream> writeToPath(
      final Path path,
      final Writer writer) throws IOException
  {
    Map<HashAlgorithm, HashingOutputStream> hashingStreams = new HashMap<>();
    try (OutputStream outputStream = new BufferedOutputStream(Files.newOutputStream(path))) {
      OutputStream os = outputStream;
      for (HashType hashType : HashType.values()) {
        os = new HashingOutputStream(hashType.getHashAlgorithm().function(), os);
        hashingStreams.put(hashType.getHashAlgorithm(), (HashingOutputStream) os);
      }
      writer.write(os);
      os.flush();
    }
    return hashingStreams;
  }

  private static Map<HashAlgorithm, HashCode> generateHashCodes(
      final Map<HashAlgorithm, HashingOutputStream> hashingStreams)
  {
    Map<HashAlgorithm, HashCode> hashCodes = new HashMap<>();
    for (Entry<HashAlgorithm, HashingOutputStream> entry : hashingStreams.entrySet()) {
      hashCodes.put(entry.getKey(), entry.getValue().hash());
    }
    return hashCodes;
  }

  private static Payload asPayload(final Path path, final String contentType) throws IOException {
    return new PathPayload(path, contentType);
  }

  public static Map<HashType, Payload> hashesToPayloads(final Map<HashAlgorithm, HashCode> hashCodes) {
    Map<HashType, Payload> payloadByHash = new EnumMap<>(HashType.class);
    for (HashType hashType : HashType.values()) {
      HashCode hashCode = hashCodes.get(hashType.getHashAlgorithm());
      if (hashCode != null) {
        Payload payload = new StringPayload(hashCode.toString(), CHECKSUM_CONTENT_TYPE);
        payloadByHash.put(hashType, payload);
      }
    }
    return payloadByHash;
  }
}
