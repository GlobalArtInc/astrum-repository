package org.globalart.astrum.repository.yum.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.security.DigestInputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.OffsetDateTime;
import java.util.HexFormat;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.repodata.PackageSource;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;
import org.globalart.astrum.repository.yum.internal.rpm.RpmParser;

final class AssetPackageSource
    implements PackageSource
{
  private final FluentAsset asset;

  private final String location;

  @Nullable
  private String checksum;

  AssetPackageSource(final FluentAsset asset, final String base) {
    this.asset = asset;
    this.location = YumPaths.location(base, asset.path());
    this.checksum = asset.blob().map(AssetBlob::checksums).map(checksums -> checksums.get("sha256")).orElse(null);
  }

  static String checksumOf(final FluentAsset asset) {
    return asset.blob().map(AssetBlob::checksums).map(checksums -> checksums.get("sha256")).orElse("");
  }

  @Override
  public String location() {
    return location;
  }

  @Override
  public String checksum() {
    return checksum == null ? "" : checksum;
  }

  @Override
  public long size() {
    return asset.blob().map(AssetBlob::blobSize).orElse(0L);
  }

  @Override
  public long fileTime() {
    return asset.blob()
        .map(AssetBlob::blobCreated)
        .or(() -> Optional.ofNullable(asset.lastUpdated()))
        .map(OffsetDateTime::toEpochSecond)
        .orElse(0L);
  }

  @Override
  public RpmPackage read() throws IOException {
    try (InputStream in = asset.download().openInputStream()) {
      if (checksum != null) {
        return RpmParser.parse(in);
      }
      MessageDigest digest = sha256();
      DigestInputStream digesting = new DigestInputStream(in, digest);
      RpmPackage rpmPackage = RpmParser.parse(digesting);
      digesting.transferTo(OutputStream.nullOutputStream());
      checksum = HexFormat.of().formatHex(digest.digest());
      return rpmPackage;
    }
  }

  private static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }
}
