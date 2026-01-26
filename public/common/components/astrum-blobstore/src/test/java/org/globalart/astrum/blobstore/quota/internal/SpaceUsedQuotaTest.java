
package org.globalart.astrum.blobstore.quota.internal;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.LIMIT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.ROOT_KEY;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SpaceUsedQuotaTest
{
  SpaceUsedQuota quota;

  @Mock
  BlobStore blobStore;

  @Mock
  BlobStoreMetrics metrics;

  @Mock
  BlobStoreConfiguration config;

  @Mock
  NestedAttributesMap attributesMap;

  @Before
  public void setup() {
    when(blobStore.getMetrics()).thenReturn(metrics);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(config);
    when(config.getName()).thenReturn("test");
    when(config.attributes(ROOT_KEY)).thenReturn(attributesMap);
    when(attributesMap.get(eq(LIMIT_KEY), eq(Number.class))).thenReturn(10L);

    quota = new SpaceUsedQuota();
  }

  @Test
  public void usingLessThanTheLimit() {
    when(metrics.getTotalSize()).thenReturn(5L);

    assertFalse(quota.check(blobStore).isViolation());
  }

  @Test
  public void usingMoreThanTheLimit() {
    when(metrics.getTotalSize()).thenReturn(20L);

    assertTrue(quota.check(blobStore).isViolation());
  }

  @Test
  public void greaterThanZeroLimitIsValid() {
    when(attributesMap.get(eq(LIMIT_KEY), eq(Number.class))).thenReturn(10L);
    quota.validateConfig(config);
  }

  @Test(expected = ValidationErrorsException.class)
  public void zeroLimitIsInvalid() {
    when(attributesMap.get(eq(LIMIT_KEY), eq(Number.class))).thenReturn(0);
    quota.validateConfig(config);
  }

  @Test(expected = IllegalArgumentException.class)
  public void noLimitIsInvalid() {
    when(attributesMap.get(eq(LIMIT_KEY), eq(Number.class))).thenReturn(null);
    quota.validateConfig(config);
  }

  @Test
  public void shouldEscape_htmlMetacharsInMessage_butNotInBlobStoreName() {
    when(config.getName()).thenReturn("poc<img src=x onerror=alert(1)>");
    when(metrics.getTotalSize()).thenReturn(20L);

    var result = quota.check(blobStore);

    assertThat("Raw HTML tag must not appear in message", result.getMessage(), not(containsString("<img")));
    assertThat("Escaped HTML entity must appear in message", result.getMessage(),
        containsString("&lt;img src=x onerror=alert(1)&gt;"));
    assertEquals("blobStoreName must carry the raw unescaped name for API consumers",
        "poc<img src=x onerror=alert(1)>", result.getBlobStoreName());
  }
}
