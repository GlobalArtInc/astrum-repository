package org.globalart.astrum.repository.yum.internal.repodata;

import javax.annotation.Nullable;

public record RepomdData(
    String type,
    String checksumType,
    String checksum,
    @Nullable String openChecksumType,
    @Nullable String openChecksum,
    String location,
    @Nullable Long timestamp,
    @Nullable Long size,
    @Nullable Long openSize,
    @Nullable String locationBase)
{
}
