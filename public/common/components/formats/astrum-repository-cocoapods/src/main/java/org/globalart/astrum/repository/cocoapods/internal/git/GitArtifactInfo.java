
package org.globalart.astrum.repository.cocoapods.internal.git;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

public class GitArtifactInfo
{
  private String host;

  private String vendor;

  private String repository;

  private String ref;

  public GitArtifactInfo(final String host, final String vendor, final String repository, @Nullable final String ref) {
    this.host = checkNotNull(host);
    this.vendor = checkNotNull(vendor);
    this.repository = checkNotNull(repository);
    this.ref = ref;
  }

  public String getHost() {
    return host;
  }

  public String getVendor() {
    return vendor;
  }

  public String getRepository() {
    return repository;
  }

  public String getRef() {
    return ref;
  }
}
