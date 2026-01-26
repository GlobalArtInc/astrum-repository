
package org.globalart.astrum.blobstore.quota.internal;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuota;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.core.IsNull.nullValue;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreQuotaServiceImplTest
{
  BlobStoreQuotaServiceImpl service;

  @Mock
  BlobStoreQuota passingQuota;

  @Mock
  BlobStoreQuota violatedQuota;

  @Mock
  BlobStore blobStore;

  @Mock
  BlobStoreConfiguration config;

  @Mock
  NestedAttributesMap attributes;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = mockStatic(QualifierUtil.class);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(config);
    when(config.attributes(eq(BlobStoreQuotaSupport.ROOT_KEY))).thenReturn(attributes);

    when(violatedQuota.check(any())).thenReturn(new BlobStoreQuotaResult(true, "test", "test"));
    when(passingQuota.check(any())).thenReturn(new BlobStoreQuotaResult(false, "test", "test"));

    when(QualifierUtil.buildQualifierBeanMap(anyList()))
        .thenReturn(Map.of("violated", violatedQuota, "passing", passingQuota));
    service = new BlobStoreQuotaServiceImpl(List.of());
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void noQuotaIsAccepted() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn(null);
    assertThat(service.checkQuota(blobStore), nullValue());
  }

  @Test
  public void passingQuota() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("passing");
    BlobStoreQuotaResult result = service.checkQuota(blobStore);
    assertThat(result, notNullValue());
    assertFalse(result.isViolation());
  }

  @Test
  public void failingQuota() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("violated");
    BlobStoreQuotaResult result = service.checkQuota(blobStore);
    assertThat(result, notNullValue());
    assertTrue(result.isViolation());
  }

  @Test
  public void missingQuota() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("non-existent");
    assertThat(service.checkQuota(blobStore), nullValue());
  }

  @Test
  public void nullQuotaTypeIsValid() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn(null);
    service.validateSoftQuotaConfig(config);
  }

  @Test(expected = ValidationErrorsException.class)
  public void emptyStringQuotaTypeFails() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("");
    service.validateSoftQuotaConfig(config);
  }

  @Test(expected = ValidationErrorsException.class)
  public void unknownQuotaTypeFails() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("TOTALLY_FAKE");
    service.validateSoftQuotaConfig(config);
  }

  @Test
  public void knownQuotaTypePasses() {
    when(attributes.get(BlobStoreQuotaSupport.TYPE_KEY, String.class)).thenReturn("passing");
    service.validateSoftQuotaConfig(config);
  }
}
