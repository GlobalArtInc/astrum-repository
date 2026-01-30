
package org.globalart.astrum.repository.internal.blobstore;

import java.util.HashMap;
import java.util.Map;

import junit.framework.TestCase;

import static org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationData.ACCESS_KEY_ID;
import static org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationData.SECRET_ACCESS_KEY;
import static org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationData.S_3;

public class BlobStoreConfigurationDataTest
    extends TestCase
{
  public static final String SOLDO_ASTRUM_PRO_BLOBSTORE = "soldo-astrum-pro-blobstore";

  public static final String SOLDO_ASTRUM_PRO_HA_BLOBSTORE = "soldo-astrum-pro-ha-blobstore";

  public static final String BUCKET = "bucket";

  public static final String REGION = "region";

  public static final String EU_WEST_1 = "eu-west-1";

  public static final String CRITICAL_ACCESS_KEY_ID = "criticalAccessKeyId";

  public static final String CRITICAL_SECRET_ACCESS_KEY = "criticalSecretAccessKey";

  public void testTestToString_NoCriticalInfoReturned() {

    BlobStoreConfigurationData blobStoreConfigurationData = new BlobStoreConfigurationData();
    blobStoreConfigurationData.setName(SOLDO_ASTRUM_PRO_BLOBSTORE);
    blobStoreConfigurationData.setType(S_3);

    Map<String, Object> s3 = new HashMap<>();
    s3.put(BUCKET, SOLDO_ASTRUM_PRO_HA_BLOBSTORE);
    s3.put(REGION, EU_WEST_1);
    s3.put(ACCESS_KEY_ID, CRITICAL_ACCESS_KEY_ID);
    s3.put(SECRET_ACCESS_KEY, CRITICAL_SECRET_ACCESS_KEY);

    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put(S_3, s3);

    blobStoreConfigurationData.setAttributes(attributes);
    String result = blobStoreConfigurationData.toString();

    assertFalse(result.contains(ACCESS_KEY_ID));
    assertFalse(result.contains(SECRET_ACCESS_KEY));
  }
}
