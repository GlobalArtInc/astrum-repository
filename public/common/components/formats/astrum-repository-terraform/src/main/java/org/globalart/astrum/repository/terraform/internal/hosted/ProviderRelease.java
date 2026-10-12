package org.globalart.astrum.repository.terraform.internal.hosted;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.internal.signing.Pgp;
import org.globalart.astrum.repository.terraform.internal.signing.Sha256Sums;

/**
 * The checks release files of a provider version pass before any of them is stored: packages against the checksums,
 * checksums against the packages the version keeps, and the signature against the checksums and the keys. Files
 * uploaded together are checked with each other, so a release that does not fit together leaves the version as it
 * was.
 */
final class ProviderRelease
{
  private ProviderRelease() {
  }

  /**
   * Release files of a version: the packages by file name with their SHA-256 digests, and the checksums, the
   * signature and the public key a publisher gave. Each of the last three may be missing.
   */
  record Files(Map<String, String> packages, @Nullable byte[] sums, @Nullable byte[] signature,
               @Nullable Pgp.PublicKey key)
  {
  }

  /**
   * @param uploaded       the files of the upload
   * @param stored         the files the version keeps; checksums and signature only when a publisher gave them
   * @param repositoryKey  the key the repository signs with, if it has one
   * @throws InvalidContentException when the files do not fit together
   */
  static void check(
      final ProviderCoordinates coordinates,
      final Files uploaded,
      final Files stored,
      @Nullable final Pgp.PublicKey repositoryKey)
  {
    String sumsName = coordinates.shasumsFileName();
    byte[] sumsDocument = uploaded.sums() != null ? uploaded.sums() : stored.sums();
    Sha256Sums sums = sumsDocument == null ? null : parse(sumsDocument);
    if (sums != null && uploaded.sums() != null) {
      stored.packages().forEach((name, sha256) -> {
        if (!uploaded.packages().containsKey(name)
            && sums.digest(name).filter(listed -> !listed.equals(sha256)).isPresent()) {
          throw new InvalidContentException(sumsName + " does not match the stored " + name);
        }
      });
    }
    if (sums != null) {
      uploaded.packages().forEach((name, sha256) -> {
        String listed = sums.digest(name).orElseThrow(() -> new InvalidContentException(sumsName
            + " does not list " + name + "; upload the checksums of the release with its packages"));
        if (!listed.equals(sha256)) {
          throw new InvalidContentException(name + " does not match its checksum in " + sumsName);
        }
      });
    }
    byte[] signature = uploaded.signature() != null ? uploaded.signature() : stored.signature();
    if (sumsDocument == null || signature == null) {
      return;
    }
    if (uploaded.key() != null) {
      if (!Pgp.verify(sumsDocument, signature, uploaded.key().armored())) {
        throw new InvalidContentException("Key " + uploaded.key().keyId() + " did not sign " + sumsName);
      }
    }
    else if (uploaded.signature() != null) {
      List<Pgp.PublicKey> keys = Stream.of(stored.key(), repositoryKey).filter(Objects::nonNull).toList();
      if (!keys.isEmpty() && keys.stream().noneMatch(key -> Pgp.verify(sumsDocument, signature, key.armored()))) {
        throw new InvalidContentException(coordinates.signatureFileName() + " is not a signature of " + sumsName
            + " by a known key");
      }
    }
  }

  private static Sha256Sums parse(final byte[] document) {
    try {
      return Sha256Sums.parse(document);
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
  }
}
