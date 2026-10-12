package org.globalart.astrum.repository.terraform;

import java.util.List;
import java.util.Locale;
import java.util.Optional;

import javax.annotation.Nullable;

/**
 * The archive types a module package can come in. Terraform hands module packages to its getter, which picks the
 * decompressor from the file extension, so a stored module keeps the extension of its type.
 */
public final class ModuleArchives
{
  public static final String TAR_GZ = "tar.gz";

  public static final String ZIP = "zip";

  private static final List<String> EXTENSIONS = List.of(TAR_GZ, "tgz", ZIP, "tar.bz2", "tbz2", "tar.xz", "txz");

  private ModuleArchives() {
  }

  /**
   * The archive type of a file name, or empty when it is not a module package.
   */
  public static Optional<String> extension(@Nullable final String fileName) {
    if (fileName == null) {
      return Optional.empty();
    }
    String lower = fileName.toLowerCase(Locale.ROOT);
    for (String extension : EXTENSIONS) {
      if (lower.endsWith("." + extension) && lower.length() > extension.length() + 1) {
        return Optional.of(canonical(extension));
      }
    }
    return Optional.empty();
  }

  /**
   * The archive type named by a getter {@code archive} parameter.
   */
  public static Optional<String> fromArchiveParameter(@Nullable final String value) {
    if (value == null) {
      return Optional.empty();
    }
    String lower = value.toLowerCase(Locale.ROOT);
    return EXTENSIONS.contains(lower) ? Optional.of(canonical(lower)) : Optional.empty();
  }

  /**
   * The archive type detected from the first bytes of a file.
   */
  public static Optional<String> sniff(final byte[] head) {
    if (head.length >= 4 && head[0] == 'P' && head[1] == 'K' && head[2] == 3 && head[3] == 4) {
      return Optional.of(ZIP);
    }
    if (head.length >= 2 && (head[0] & 0xff) == 0x1f && (head[1] & 0xff) == 0x8b) {
      return Optional.of(TAR_GZ);
    }
    if (head.length >= 3 && head[0] == 'B' && head[1] == 'Z' && head[2] == 'h') {
      return Optional.of("tar.bz2");
    }
    if (head.length >= 6 && (head[0] & 0xff) == 0xfd && head[1] == '7' && head[2] == 'z' && head[3] == 'X'
        && head[4] == 'Z' && head[5] == 0) {
      return Optional.of("tar.xz");
    }
    return Optional.empty();
  }

  private static String canonical(final String extension) {
    return switch (extension) {
      case "tgz" -> TAR_GZ;
      case "tbz2" -> "tar.bz2";
      case "txz" -> "tar.xz";
      default -> extension;
    };
  }
}
