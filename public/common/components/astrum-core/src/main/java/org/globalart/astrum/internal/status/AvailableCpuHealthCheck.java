
package org.globalart.astrum.internal.status;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Health check that indicates if the available JVM reported CPU count is below the recommended threshold.
 * <p>
 * The {@see Runtime#availableProcessors} API used to check the number of available processors may return different
 * results during the lifetime of the JVM, but we will always consider it not healthy if the CPU is dynamically reduced
 * below threshold.
 *
 * @since 3.17
 */
@Component
@Qualifier("Available CPUs")
public class AvailableCpuHealthCheck
    extends HealthCheckComponentSupport
{
  /**
   * Minimum recommended CPU Count
   */
  static final int MIN_RECOMMENDED_CPU_COUNT = 2;

  private static final String HEALTHY_MESSAGE =
      "The host system is allocating a maximum of %d cores to the application.";

  private static final String UNHEALTHY_MESSAGE = HEALTHY_MESSAGE + " A minimum of %d is recommended.";

  private int minCpuCount;

  public AvailableCpuHealthCheck() {
    this(MIN_RECOMMENDED_CPU_COUNT);
  }

  @VisibleForTesting
  AvailableCpuHealthCheck(final int minCpuCount) {
    this.minCpuCount = minCpuCount;
  }

  private Runtime getRuntime() {
    return Runtime.getRuntime();
  }

  @Override
  protected Result check() {
    int available = getRuntime().availableProcessors();

    if (minCpuCount > available) {
      return Result.unhealthy(UNHEALTHY_MESSAGE, available, minCpuCount);
    }

    return Result.healthy(HEALTHY_MESSAGE, available);
  }
}
