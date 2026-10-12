package org.globalart.astrum.repository.terraform;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.terraform.TerraformPaths.PROVIDERS;

/**
 * Address of a provider inside a repository: {@code namespace/type}, optionally at one version.
 */
public record ProviderCoordinates(String namespace, String type, @Nullable String version)
{
  public static final String SHA256SUMS = "SHA256SUMS";

  public static final String SIGNATURE_EXTENSION = ".sig";

  public static final String KEY_FILE_NAME = "signing-key.asc";

  public ProviderCoordinates {
    checkArgument(TerraformPaths.isValidName(namespace), "Invalid provider namespace: %s", namespace);
    checkArgument(TerraformPaths.isValidName(type), "Invalid provider type: %s", type);
    checkArgument(version == null || TerraformPaths.isValidVersion(version), "Invalid provider version: %s", version);
  }

  /**
   * The provider address without host.
   */
  public String address() {
    return namespace + "/" + type;
  }

  /**
   * The name of the provider version component, whose namespace is the provider namespace: the type.
   */
  public String componentName() {
    return type;
  }

  public String basePath() {
    return PROVIDERS + address();
  }

  public String versionsPath() {
    return basePath() + "/versions";
  }

  public String directory() {
    return basePath() + "/" + checkNotNull(version);
  }

  public String downloadPath(final Platform platform) {
    return directory() + "/download/" + platform.os() + "/" + platform.arch();
  }

  /**
   * Prefix shared by the release files of this version, {@code terraform-provider-<type>_<version>_}.
   */
  public String filePrefix() {
    return "terraform-provider-" + type + "_" + checkNotNull(version) + "_";
  }

  public String archiveFileName(final Platform platform) {
    return filePrefix() + platform.os() + "_" + platform.arch() + ".zip";
  }

  public String shasumsFileName() {
    return filePrefix() + SHA256SUMS;
  }

  public String signatureFileName() {
    return shasumsFileName() + SIGNATURE_EXTENSION;
  }

  public String manifestFileName() {
    return filePrefix() + "manifest.json";
  }

  public String filePath(final String fileName) {
    return directory() + "/" + fileName;
  }
}
