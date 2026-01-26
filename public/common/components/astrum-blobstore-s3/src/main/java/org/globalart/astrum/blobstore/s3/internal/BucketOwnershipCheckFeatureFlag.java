
package org.globalart.astrum.blobstore.s3.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.FeatureFlags;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
public class BucketOwnershipCheckFeatureFlag
{
  private final boolean isDisabled;

  @Autowired
  public BucketOwnershipCheckFeatureFlag(
      @Value(FeatureFlags.BLOBSTORE_OWNERSHIP_CHECK_DISABLED_NAMED_VALUE) final Boolean isDisabled)
  {
    this.isDisabled = Boolean.TRUE.equals(isDisabled);
  }

  public boolean isDisabled() {
    return isDisabled;
  }
}
