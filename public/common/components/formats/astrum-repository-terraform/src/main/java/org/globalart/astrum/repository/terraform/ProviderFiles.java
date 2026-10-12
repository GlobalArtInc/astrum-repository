package org.globalart.astrum.repository.terraform;

import java.util.Locale;
import java.util.Optional;

import static org.globalart.astrum.repository.terraform.ProviderCoordinates.KEY_FILE_NAME;
import static org.globalart.astrum.repository.terraform.ProviderCoordinates.SHA256SUMS;
import static org.globalart.astrum.repository.terraform.ProviderCoordinates.SIGNATURE_EXTENSION;

/**
 * The release files of one provider version, told apart by their names as release tooling writes them:
 * {@code terraform-provider-<type>_<version>_<os>_<arch>.zip}, {@code ..._SHA256SUMS},
 * {@code ..._SHA256SUMS.sig} (also {@code ..._SHA256SUMS.<key id>.sig}), {@code ..._manifest.json}, and the
 * public key that signed the checksums.
 */
public final class ProviderFiles
{
  private ProviderFiles() {
  }

  public static Optional<AssetKind> kindOf(final ProviderCoordinates coordinates, final String fileName) {
    if (KEY_FILE_NAME.equals(fileName)) {
      return Optional.of(AssetKind.PROVIDER_KEY);
    }
    String prefix = coordinates.filePrefix();
    if (!fileName.startsWith(prefix)) {
      return Optional.empty();
    }
    String rest = fileName.substring(prefix.length());
    if (platformOf(coordinates, fileName).isPresent()) {
      return Optional.of(AssetKind.PROVIDER_ARCHIVE);
    }
    if (SHA256SUMS.equals(rest)) {
      return Optional.of(AssetKind.PROVIDER_SHASUMS);
    }
    if (rest.startsWith(SHA256SUMS + ".") && rest.endsWith(SIGNATURE_EXTENSION)
        && rest.indexOf('/') < 0) {
      return Optional.of(AssetKind.PROVIDER_SIGNATURE);
    }
    if ("manifest.json".equals(rest)) {
      return Optional.of(AssetKind.PROVIDER_MANIFEST);
    }
    return Optional.empty();
  }

  /**
   * The platform of a provider package named {@code terraform-provider-<type>_<version>_<os>_<arch>.zip}.
   */
  public static Optional<Platform> platformOf(final ProviderCoordinates coordinates, final String fileName) {
    String prefix = coordinates.filePrefix();
    if (!fileName.startsWith(prefix) || !fileName.endsWith(".zip")) {
      return Optional.empty();
    }
    String platform = fileName.substring(prefix.length(), fileName.length() - ".zip".length());
    int separator = platform.indexOf('_');
    if (separator <= 0 || separator != platform.lastIndexOf('_')) {
      return Optional.empty();
    }
    String os = platform.substring(0, separator);
    String arch = platform.substring(separator + 1);
    if (!TerraformPaths.isValidPlatformPart(os) || !TerraformPaths.isValidPlatformPart(arch)) {
      return Optional.empty();
    }
    return Optional.of(new Platform(os, arch));
  }

  /**
   * Whether an uploaded file is a public key, by the extensions key exports use.
   */
  public static boolean isKeyUpload(final String fileName) {
    String lower = fileName.toLowerCase(Locale.ROOT);
    return KEY_FILE_NAME.equals(fileName) || lower.endsWith(".asc") || lower.endsWith(".gpg")
        || lower.endsWith(".pub") || lower.endsWith(".key");
  }
}
