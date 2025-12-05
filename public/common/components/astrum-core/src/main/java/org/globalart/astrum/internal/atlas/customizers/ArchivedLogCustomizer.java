
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.time.LocalDate;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.internal.atlas.SupportZipGeneratorImpl;
import org.globalart.astrum.supportzip.FileContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.OPTIONAL;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.ARCHIVEDLOG;
import org.springframework.stereotype.Component;

@Component
public class ArchivedLogCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ApplicationDirectories applicationDirectories;

  @Autowired
  public ArchivedLogCustomizer(final ApplicationDirectories applicationDirectories) {
    this.applicationDirectories = checkNotNull(applicationDirectories);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    int archivedLogSize = SupportZipGeneratorImpl.getArchivedLogSize();
    includeArchivedLogs(supportBundle, archivedLogSize);
  }

  private void includeFileIfExists(
      final SupportBundle supportBundle,
      final File file,
      final String prefix,
      final Priority priority)
  {
    if (file != null && file.exists()) {
      log.debug("Including file: {}", file);
      supportBundle.add(
          new FileContentSourceSupport(ARCHIVEDLOG, String.format("%s/%s", prefix, file.getName()), file, priority));
    }
    else {
      log.debug("Skipping non-existent file: {}", file);
    }
  }

  private void includeArchivedLogs(final SupportBundle supportBundle, final int archivedLogSize) {
    List<String> archivedLogList = archivedLogFormatter(archivedLogSize);
    for (String log : archivedLogList) {
      includeFileIfExists(supportBundle, new File(applicationDirectories.getWorkDirectory(), log), "log/archived-logs/",
          OPTIONAL);
    }
  }

  private List<String> archivedLogFormatter(int archivedLogSize) {
    ArrayList<String> archivedLogList = new ArrayList<String>();
    String LOGEXT = ".log.gz";
    LocalDate currentDate = LocalDate.now();
    DateTimeFormatter formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd");

    for (int i = 0; i <= archivedLogSize; i++) {
      StringBuilder requestLog = new StringBuilder("log/request-");
      StringBuilder auditLog = new StringBuilder("log/audit/audit-");
      StringBuilder astrumLog = new StringBuilder("log/astrum-");
      // need to take the current date and subtract i days from it
      LocalDate previousDate = currentDate.minusDays(i);
      String formattedPreviousDate = previousDate.format(formatter);

      requestLog.append(formattedPreviousDate).append(LOGEXT);
      auditLog.append(formattedPreviousDate).append(LOGEXT);
      astrumLog.append(formattedPreviousDate).append(LOGEXT);

      archivedLogList.add(requestLog.toString());
      archivedLogList.add(auditLog.toString());
      archivedLogList.add(astrumLog.toString());
    }
    return archivedLogList;
  }

}
