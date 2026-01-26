
package org.globalart.astrum.blobstore.metrics;

import java.util.Map;

import org.globalart.astrum.blobstore.BlobSupport;
import org.globalart.astrum.blobstore.api.BlobMetrics;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.reflect.MethodSignature;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreAnalyticsAspectTest
{

  private BlobStoreAnalyticsAspect aspect;

  @Mock
  private ProceedingJoinPoint joinPoint;

  @Mock
  private MethodSignature methodSignature;

  @Mock
  private BlobStore blobStore;

  @Mock
  private OperationMetrics operationMetrics;

  @Mock
  private BlobMetrics blobMetrics;

  @Mock
  private BlobSupport blobSupport;

  @Before
  public void setUp() {
    aspect = new BlobStoreAnalyticsAspect();
  }

  @Test
  public void testMonitorBlobStoreOperation_SuccessfulRequest() throws Throwable {
    // Mock behavior
    when(joinPoint.getTarget()).thenReturn(blobStore);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(TestBlobStore.class.getMethod("uploadBlob"));
    when(blobStore.getOperationMetricsDelta()).thenReturn(Map.of(OperationType.UPLOAD, operationMetrics));
    when(joinPoint.proceed()).thenReturn(blobSupport);
    when(blobSupport.getMetrics()).thenReturn(blobMetrics);
    when(blobSupport.getMetrics().getContentSize()).thenReturn(100L);

    // Execute the aspect
    Object result = aspect.monitorBlobStoreOperation(joinPoint,
        TestBlobStore.class.getMethod("uploadBlob").getAnnotation(MonitoringBlobStoreMetrics.class));

    // Verify interactions
    verify(operationMetrics).addSuccessfulRequest();
    verify(operationMetrics).addTimeOnRequests(anyLong());
    verify(operationMetrics).addBlobSize(100L);
    verify(joinPoint).proceed();
    assertEquals(blobSupport, result);
  }

  @Test
  public void testMonitorBlobStoreOperation_ErrorRequest() throws Throwable {
    // Mock behavior
    when(joinPoint.getTarget()).thenReturn(blobStore);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(TestBlobStore.class.getMethod("uploadBlob"));
    when(blobStore.getOperationMetricsDelta()).thenReturn(Map.of(OperationType.UPLOAD, operationMetrics));
    when(joinPoint.proceed()).thenThrow(new RuntimeException("Test Exception"));

    // Execute the aspect and expect an exception
    try {
      aspect.monitorBlobStoreOperation(joinPoint,
          TestBlobStore.class.getMethod("uploadBlob").getAnnotation(MonitoringBlobStoreMetrics.class));
      fail("Expected exception was not thrown");
    }
    catch (RuntimeException e) {
      assertEquals("Test Exception", e.getMessage());
    }

    // Verify interactions
    verify(operationMetrics).addErrorRequest();
    verify(joinPoint).proceed();
  }

  // Internal test class with annotated method
  private static class TestBlobStore
  {

    @MonitoringBlobStoreMetrics(operationType = OperationType.UPLOAD)
    public void uploadBlob() {
    }
  }
}
