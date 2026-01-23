
package org.globalart.astrum.blobstore.metrics;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import org.globalart.astrum.blobstore.api.OperationType;

/**
 * Marks a blob store method with {@link OperationType} type to collect metrics.
 *
 * @since 3.38
 */
@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.METHOD})
public @interface MonitoringBlobStoreMetrics
{
  OperationType operationType();
}
