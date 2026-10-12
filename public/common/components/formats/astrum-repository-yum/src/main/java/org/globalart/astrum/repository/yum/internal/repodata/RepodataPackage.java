package org.globalart.astrum.repository.yum.internal.repodata;

import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

public record RepodataPackage(RpmPackage rpm, String location, String checksum, long packageSize, long fileTime)
{
}
