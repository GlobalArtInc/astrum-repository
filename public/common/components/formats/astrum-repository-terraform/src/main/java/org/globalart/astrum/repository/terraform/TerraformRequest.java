package org.globalart.astrum.repository.terraform;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Context;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * What a request addresses, as decided by the matchers of {@link TerraformPaths}.
 */
public record TerraformRequest(
    Endpoint endpoint,
    @Nullable ModuleCoordinates module,
    @Nullable ProviderCoordinates provider,
    @Nullable Platform platform,
    @Nullable String file,
    @Nullable String hostname,
    @Nullable AssetKind assetKind)
{
  public enum Endpoint
  {
    DISCOVERY,
    TOKEN,
    MODULE_VERSIONS,
    MODULE_DOWNLOAD,
    MODULE_ARCHIVE,
    PROVIDER_VERSIONS,
    PROVIDER_DOWNLOAD,
    PROVIDER_FILE,
    MIRROR_INDEX,
    MIRROR_VERSION
  }

  public static TerraformRequest of(final Endpoint endpoint) {
    return new TerraformRequest(endpoint, null, null, null, null, null, null);
  }

  public static TerraformRequest moduleVersions(final ModuleCoordinates module) {
    return new TerraformRequest(Endpoint.MODULE_VERSIONS, module, null, null, null, null, AssetKind.MODULE_VERSIONS);
  }

  public static TerraformRequest moduleDownload(final ModuleCoordinates module) {
    return new TerraformRequest(Endpoint.MODULE_DOWNLOAD, module, null, null, null, null, null);
  }

  public static TerraformRequest moduleArchive(final ModuleCoordinates module, final String file) {
    return new TerraformRequest(Endpoint.MODULE_ARCHIVE, module, null, null, file, null, AssetKind.MODULE_ARCHIVE);
  }

  public static TerraformRequest providerVersions(final ProviderCoordinates provider) {
    return new TerraformRequest(Endpoint.PROVIDER_VERSIONS, null, provider, null, null, null,
        AssetKind.PROVIDER_VERSIONS);
  }

  public static TerraformRequest providerDownload(final ProviderCoordinates provider, final Platform platform) {
    return new TerraformRequest(Endpoint.PROVIDER_DOWNLOAD, null, provider, platform, null, null,
        AssetKind.PROVIDER_DOWNLOAD);
  }

  public static TerraformRequest providerFile(final ProviderCoordinates provider, final String file,
                                              final AssetKind kind)
  {
    return new TerraformRequest(Endpoint.PROVIDER_FILE, null, provider, ProviderFiles.platformOf(provider, file)
        .orElse(null), file, null, kind);
  }

  public static TerraformRequest mirrorIndex(final String hostname, final ProviderCoordinates provider) {
    return new TerraformRequest(Endpoint.MIRROR_INDEX, null, provider, null, null, hostname, null);
  }

  public static TerraformRequest mirrorVersion(final String hostname, final ProviderCoordinates provider) {
    return new TerraformRequest(Endpoint.MIRROR_VERSION, null, provider, null, null, hostname, null);
  }

  public static TerraformRequest of(final Context context) {
    return checkNotNull(context.getAttributes().get(TerraformRequest.class), "Not a Terraform request");
  }

  /**
   * The path of the asset that stores what this request addresses, for the endpoints that are stored.
   */
  public String assetPath() {
    return switch (endpoint) {
      case MODULE_VERSIONS -> checkNotNull(module).versionsPath();
      case MODULE_ARCHIVE -> checkNotNull(module).directory() + "/" + file;
      case PROVIDER_VERSIONS -> checkNotNull(provider).versionsPath();
      case PROVIDER_DOWNLOAD -> checkNotNull(provider).downloadPath(checkNotNull(platform));
      case PROVIDER_FILE -> checkNotNull(provider).filePath(checkNotNull(file));
      default -> throw new IllegalStateException("Endpoint " + endpoint + " is not stored");
    };
  }
}
