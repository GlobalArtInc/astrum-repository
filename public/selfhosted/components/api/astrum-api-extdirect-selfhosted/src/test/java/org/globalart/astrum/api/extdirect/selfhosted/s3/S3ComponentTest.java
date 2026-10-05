
package org.globalart.astrum.api.extdirect.selfhosted.s3;

import java.util.List;

import org.globalart.astrum.api.extdirect.selfhosted.s3.model.S3EncryptionTypeXO;
import org.globalart.astrum.api.extdirect.selfhosted.s3.model.S3RegionXO;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@ExtendWith(AuthenticationExtension.class)
@WithUser
public class S3ComponentTest
{
  private final S3Component underTest = new S3Component();

  @Test
  public void testRegions() throws Exception {
    List<S3RegionXO> regions = underTest.regions();
    assertRegion(regions.get(0), 0, "DEFAULT", "Default");
  }

  @Test
  public void testEncryptionTypes() throws Exception {
    List<S3EncryptionTypeXO> encryptionTypes = underTest.encryptionTypes();
    assertEncryptionType(encryptionTypes.get(0), 0, "none", "None");
    assertEncryptionType(encryptionTypes.get(1), 1, "s3ManagedEncryption", "S3 Managed Encryption");
    assertEncryptionType(encryptionTypes.get(2), 2, "kmsManagedEncryption", "KMS Managed Encryption");
  }

  private void assertRegion(final S3RegionXO region, final int order, final String id, final String name) {
    assertThat(region.getOrder(), is(order));
    assertThat(region.getId(), is(id));
    assertThat(region.getName(), is(name));
  }

  private void assertEncryptionType(
      final S3EncryptionTypeXO encryptionType,
      final int order,
      final String id,
      final String name)
  {
    assertThat(encryptionType.getOrder(), is(order));
    assertThat(encryptionType.getId(), is(id));
    assertThat(encryptionType.getName(), is(name));
  }
}
