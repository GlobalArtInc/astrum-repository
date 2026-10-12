package org.globalart.astrum.repository.docker.internal.v2;

import java.util.List;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.HashCode;
import com.google.common.hash.Hashing;

public final class DockerDigest
{
  public static final String SHA256 = "sha256";

  public static final String SHA512 = "sha512";

  private static final Pattern DIGEST = Pattern.compile("^[a-z0-9]+(?:[.+_-][a-z0-9]+)*:[a-zA-Z0-9=_-]+$");

  private static final Pattern SHA256_DIGEST = Pattern.compile("^sha256:[a-f0-9]{64}$");

  private static final Pattern SHA512_DIGEST = Pattern.compile("^sha512:[a-f0-9]{128}$");

  private static final List<HashAlgorithm> SHA256_HASHES = List.of(HashAlgorithm.SHA1, HashAlgorithm.SHA256);

  private static final List<HashAlgorithm> SHA512_HASHES =
      List.of(HashAlgorithm.SHA1, HashAlgorithm.SHA256, HashAlgorithm.SHA512);

  private DockerDigest() {
  }

  public static boolean isDigest(@Nullable final String value) {
    return value != null && DIGEST.matcher(value).matches();
  }

  public static boolean isSupported(@Nullable final String value) {
    return value != null && (SHA256_DIGEST.matcher(value).matches() || SHA512_DIGEST.matcher(value).matches());
  }

  public static boolean isSha512(@Nullable final String value) {
    return value != null && value.startsWith(SHA512 + ":");
  }

  public static String sha256(final byte[] bytes) {
    return SHA256 + ":" + Hashing.sha256().hashBytes(bytes).toString();
  }

  public static String sha256(final HashCode hashCode) {
    return SHA256 + ":" + hashCode.toString();
  }

  /**
   * The digest of the bytes in the algorithm of the given digest, sha256 when it names no supported algorithm.
   */
  public static String of(final byte[] bytes, @Nullable final String like) {
    if (isSha512(like)) {
      return SHA512 + ":" + Hashing.sha512().hashBytes(bytes).toString();
    }
    return sha256(bytes);
  }

  /**
   * The hashes to compute for content that must match the given digest.
   */
  public static List<HashAlgorithm> hashesFor(@Nullable final String digest) {
    return isSha512(digest) ? SHA512_HASHES : SHA256_HASHES;
  }

  /**
   * The digest of an ingested blob in the algorithm of the given digest.
   */
  @Nullable
  public static String of(final TempBlob blob, final String like) {
    if (isSha512(like)) {
      HashCode hash = blob.getHashes().get(HashAlgorithm.SHA512);
      return hash == null ? null : SHA512 + ":" + hash;
    }
    HashCode hash = blob.getHashes().get(HashAlgorithm.SHA256);
    return hash == null ? null : sha256(hash);
  }

  public static String hex(final String digest) {
    int index = digest.indexOf(':');
    return index < 0 ? digest : digest.substring(index + 1);
  }
}
