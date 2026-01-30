
package org.globalart.astrum.repository.internal.blobstore;

import java.util.Arrays;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;

import static java.util.Collections.singletonList;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.internal.blobstore.BlobStoreOverrideImpl.ASTRUM_BLOB_STORE_OVERRIDE;

/**
 * Test {@code BlobStoreOverrideImpl} with various override strings in a situation where only the default blob store
 * exists and the override string is not expected to trigger an update the the blob store configuration.
 */
@RunWith(Parameterized.class)
public class BlobStoreOverrideImplParsingTest
    extends BlobStoreOverrideImplTestSupport
{
  @Parameter
  public String envString;

  @Parameters(name = "ASTRUM_BLOB_STORE_OVERRIDE: {0}")
  public static Iterable<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {null},
        {"{}"},
        {""},
        {"null"},
        {"{\"default\":{\"file\":{\"path\":\"default\"}}}"},
        {"{\"default\":{\"file\":{}}}"},
        {"{\"default\":{\"file\":null}}"},
        {"{\"default\":{}}"},
        {"{\"default\":null}"},
        {"{\"wrong name\":{\"file\":{\"path\":\"other_path\"}}}"},
        {"{\"default\":{\"s3\":{\"path\":\"default\"}}}"},
        {"{\"one\":null, \"two\":null}"},
        {"{\"one\":{}, \"two\":{}}"},
    });
  }

  @Test
  public void testParseWithNoUpdate() {
    environmentVariables.set(ASTRUM_BLOB_STORE_OVERRIDE, envString);
    BlobStoreConfiguration config = defaultConfig();
    when(configStore.list()).thenReturn(singletonList(config));
    underTest.apply();
    verify(configStore, never()).update(any());
  }
}
