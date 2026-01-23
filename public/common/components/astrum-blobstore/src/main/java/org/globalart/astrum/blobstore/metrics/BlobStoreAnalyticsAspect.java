
package org.globalart.astrum.blobstore.metrics;

import org.globalart.astrum.blobstore.BlobSupport;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkState;

/**
 * Aspect to monitor blob store operations (see {@link OperationType}) of the annotated method.
 *
 */
@Aspect
public class BlobStoreAnalyticsAspect
{
  @Around("@annotation(metricsAnnotation) && execution(* *(..))")
  public Object monitorBlobStoreOperation(
      final ProceedingJoinPoint joinPoint,
      final MonitoringBlobStoreMetrics metricsAnnotation) throws Throwable
  {
    String clazz = joinPoint.getTarget().getClass().getSimpleName();
    String methodName = joinPoint.getSignature().getName();

    checkState(metricsAnnotation != null);
    OperationType operationType = metricsAnnotation.operationType();

    OperationMetrics operationMetrics = null;
    if (joinPoint.getTarget() instanceof BlobStore blobStore) {
      blobStore = (BlobStore) joinPoint.getTarget();
      operationMetrics = blobStore.getOperationMetricsDelta().get(operationType);
    }
    if (operationMetrics == null) {
      LoggerFactory.getLogger(BlobStoreAnalyticsAspect.class)
          .info("Can't monitor operation metrics for class={}, methodName={}", clazz, methodName);
      return joinPoint.proceed();
    }

    long start = System.currentTimeMillis();
    try {
      Object result = joinPoint.proceed();

      // Record metrics only in case of successful processing.
      operationMetrics.addSuccessfulRequest();
      operationMetrics.addTimeOnRequests(System.currentTimeMillis() - start);

      if (result instanceof BlobSupport) {
        long totalSize = ((BlobSupport) result).getMetrics().getContentSize();
        operationMetrics.addBlobSize(totalSize);
      }

      return result;
    }
    catch (Exception e) {
      operationMetrics.addErrorRequest();
      throw e;
    }
  }
}
