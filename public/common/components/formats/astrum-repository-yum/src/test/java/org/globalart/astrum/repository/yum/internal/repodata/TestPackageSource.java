package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.security.MessageDigest;
import java.util.HexFormat;

import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;
import org.globalart.astrum.repository.yum.internal.rpm.RpmParser;

public record TestPackageSource(String location, byte[] bytes, long fileTime)
    implements PackageSource
{
  @Override
  public String checksum() {
    try {
      return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
    }
    catch (Exception e) {
      throw new IllegalStateException(e);
    }
  }

  @Override
  public long size() {
    return bytes.length;
  }

  @Override
  public RpmPackage read() throws IOException {
    return RpmParser.parse(new ByteArrayInputStream(bytes));
  }
}
