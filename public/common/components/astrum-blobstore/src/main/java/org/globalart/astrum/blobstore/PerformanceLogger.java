
package org.globalart.astrum.blobstore;

import java.io.InputStream;

import org.slf4j.LoggerFactory;
import org.globalart.astrum.blobstore.api.Blob;

import org.slf4j.Logger;

import static java.lang.String.format;

/**
 * Logs blob store performance statistics.
 *
 * @since 3.21
 */
public class PerformanceLogger
{

  private static final String IOSTAT_LOGGER_NAME = "org.globalart.astrum.blobstore.iostat";

  private final Logger log = LoggerFactory.getLogger(IOSTAT_LOGGER_NAME);

  private String blobStoreName = "<not set>";

  public void setBlobStoreName(final String blobStoreName) {
    this.blobStoreName = blobStoreName;
  }

  public InputStream maybeWrapForPerformanceLogging(final InputStream inputStream) {
    if (log.isDebugEnabled()) {
      return new PerformanceLoggingInputStream(inputStream, this);
    }
    else {
      return inputStream;
    }
  }

  public void logRead(final long bytes, final long nanos) {
    if (!log.isDebugEnabled()) {
      return;
    }

    double millis = 0d;
    double mbPerSecond = Double.NaN;
    if (nanos > 0) {
      millis = ((double) nanos) / 1e6d;
      mbPerSecond = ((double) bytes) / ((double) nanos) * 1e3d;
    }
    log.debug(format("blobstore %s: %d bytes read in %g ms (%g mb/s)", blobStoreName, bytes, millis, mbPerSecond));
  }

  public void logCreate(final Blob blob, final long nanos) {
    if (!log.isDebugEnabled()) {
      return;
    }

    long bytes = blob.getMetrics().getContentSize();
    double millis = 0d;
    double mbPerSecond = Double.NaN;
    if (nanos > 0) {
      millis = ((double) nanos) / 1e6d;
      mbPerSecond = ((double) bytes) / ((double) nanos) * 1e3d;
    }
    log.debug(format("blobstore %s: %d bytes written in %g ms (%g mb/s)", blobStoreName, bytes, millis, mbPerSecond));
  }

  public void logDelete(final long nanos) {
    if (!log.isDebugEnabled()) {
      return;
    }

    double millis = 0d;
    if (nanos > 0) {
      millis = ((double) nanos) / 1e6d;
    }
    log.debug(format("blobstore %s: blob deleted in %g ms", blobStoreName, millis));
  }
}
