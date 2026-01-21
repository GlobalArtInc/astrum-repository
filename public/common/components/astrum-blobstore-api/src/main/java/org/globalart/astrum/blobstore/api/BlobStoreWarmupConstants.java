
package org.globalart.astrum.blobstore.api;

/**
 * Shared constants for blob store warmup configuration.
 *
 * These constants are used by all blob store implementations (S3, Azure, etc.) to
 * configure warmup behavior via system properties.
 */
public final class BlobStoreWarmupConstants
{
  private BlobStoreWarmupConstants() {
    // Utility class - no instantiation
  }

  /**
   * System property key for warmup timeout in milliseconds.
   * Default: 10000ms (10 seconds)
   */
  public static final String WARMUP_TIMEOUT_PROPERTY = "astrum.blobstore.warmup.timeout.ms";

  /**
   * Default warmup timeout in milliseconds.
   */
  public static final long DEFAULT_WARMUP_TIMEOUT_MS = 10000L;

  /**
   * System property key for slow warmup threshold in milliseconds.
   * If warmup takes longer than this threshold, a warning is logged.
   * Default: 2000ms (2 seconds)
   */
  public static final String WARMUP_SLOW_THRESHOLD_PROPERTY = "astrum.blobstore.warmup.slow.threshold.ms";

  /**
   * Default slow warmup threshold in milliseconds.
   */
  public static final long DEFAULT_SLOW_THRESHOLD_MS = 2000L;

  /**
   * Maximum number of warmup retry attempts.
   */
  public static final int WARMUP_MAX_ATTEMPTS = 3;

  /**
   * Time-to-live for warmup failure state in milliseconds.
   * After this duration, blob store operations proceed normally even if warmup failed.
   * Default: 300000ms (5 minutes)
   */
  public static final long WARMUP_FAILURE_TTL_MS = 300000L;
}
