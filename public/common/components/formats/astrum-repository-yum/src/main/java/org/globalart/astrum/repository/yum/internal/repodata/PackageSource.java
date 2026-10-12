package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;

import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

public interface PackageSource
{
  String location();

  String checksum();

  long size();

  long fileTime();

  RpmPackage read() throws IOException;
}
