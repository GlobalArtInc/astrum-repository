
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.internal.support.DbDiagnostics;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import org.apache.commons.io.FileUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.HIGH;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.DBINFO;
import org.springframework.stereotype.Component;

/**
 * Creates and adds db info file to support bundle.
 *
 */
@Component
public class DbInfoCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DbDiagnostics dbDiagnostics;

  @Autowired
  public DbInfoCustomizer(final DbDiagnostics dbDiagnostics) {
    this.dbDiagnostics = checkNotNull(dbDiagnostics);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    supportBundle.add(new GeneratedContentSourceSupport(DBINFO, "info/dbFileInfo.txt", HIGH)
    {
      @Override
      protected void generate(final File file) throws IOException {
        String dbInfoOutput = dbDiagnostics.getDbFileInfo();
        FileUtils.write(file, dbInfoOutput, StandardCharsets.UTF_8);
      }
    });
  }
}
