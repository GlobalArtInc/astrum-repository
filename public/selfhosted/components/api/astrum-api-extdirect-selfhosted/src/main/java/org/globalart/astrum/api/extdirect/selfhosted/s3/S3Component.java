
package org.globalart.astrum.api.extdirect.selfhosted.s3;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.globalart.astrum.blobstore.s3.internal.AmazonS3Factory;
import org.globalart.astrum.blobstore.s3.internal.encryption.KMSEncrypter;
import org.globalart.astrum.blobstore.s3.internal.encryption.NoEncrypter;
import org.globalart.astrum.blobstore.s3.internal.encryption.S3ManagedEncrypter;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.api.extdirect.selfhosted.s3.model.S3EncryptionTypeXO;
import org.globalart.astrum.api.extdirect.selfhosted.s3.model.S3RegionXO;

import software.amazon.awssdk.regions.Region;
import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

/**
 * S3 {@link DirectComponent}.
 *
 * @since 3.12
 */
@Component
@DirectAction(action = "s3_S3")
public class S3Component
    extends DirectComponentSupport
{
  private static final String DEFAULT_LABEL = "Default";

  private final List<S3RegionXO> regions;

  private final List<S3EncryptionTypeXO> encryptionTypes;

  public S3Component() {
    regions = new ArrayList<>();
    regions.add(new S3RegionXO().withOrder(0).withId(AmazonS3Factory.DEFAULT).withName(DEFAULT_LABEL));

    // Add all AWS SDK 2.x regions
    var regionValues = Region.regions();
    int index = 0;
    for (Region region : regionValues) {
      regions.add(new S3RegionXO()
          .withOrder(index + 1)
          .withId(region.id())
          .withName(region.id()));
      index++;
    }

    this.encryptionTypes = Arrays.asList(
        new S3EncryptionTypeXO().withOrder(0).withId(NoEncrypter.ID).withName(NoEncrypter.NAME),
        new S3EncryptionTypeXO().withOrder(1).withId(S3ManagedEncrypter.ID).withName(S3ManagedEncrypter.NAME),
        new S3EncryptionTypeXO().withOrder(2).withId(KMSEncrypter.ID).withName(KMSEncrypter.NAME));
  }

  public List<S3RegionXO> getRegions() {
    return regions;
  }

  public List<S3EncryptionTypeXO> getEncryptionTypes() {
    return encryptionTypes;
  }

  /**
   * S3 regions
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public List<S3RegionXO> regions() {
    return regions;
  }

  /**
   * S3 encryption types
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public List<S3EncryptionTypeXO> encryptionTypes() {
    return encryptionTypes;
  }
}
