
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import org.apache.commons.io.FileUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.LOW;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.AUDITLOG;
import org.springframework.stereotype.Component;

/**
 * Adds audit log files to support bundle.
 *
 * @since 3.16
 */
@Component
public class AuditLogCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LogManager logManager;

  @Autowired
  public AuditLogCustomizer(final LogManager logManager) {
    this.logManager = checkNotNull(logManager);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    // add source for astrum.log
    supportBundle.add(new GeneratedContentSourceSupport(AUDITLOG, "log/audit.log", LOW)
    {
      @Override
      protected void generate(final File file) {
        try {
          // didn't bother with try-with-resources as FileUtils.copyInputStreamToFile handles closing the input stream
          InputStream is = logManager.getLogFileStream("audit.log", 0, Long.MAX_VALUE);
          if (is != null) {
            FileUtils.copyInputStreamToFile(is, file);
          }
          else {
            log.debug("Not including missing audit.log file");
          }
        }
        catch (IOException e) {
          log.debug("Unable to include audit.log file", e);
        }
      }
    });
  }
}
