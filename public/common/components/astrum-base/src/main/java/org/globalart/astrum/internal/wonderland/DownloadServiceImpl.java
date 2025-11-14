
package org.globalart.astrum.internal.wonderland;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.concurrent.atomic.AtomicLong;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.common.wonderland.DownloadService;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link DownloadService}.
 *
 * @since 2.8
 */
@Component
@Qualifier("default")
public class DownloadServiceImpl
    implements DownloadService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /**
   * Counter used to generate unique names.
   */
  private static final AtomicLong counter = new AtomicLong();

  /**
   * Directory where files to be downloaded are stored.
   */
  private final File downloadDir;

  private final AuthTicketService authTickets;

  @Autowired
  public DownloadServiceImpl(
      final ApplicationDirectories applicationDirectories,
      final AuthTicketService authTicketService)
  {
    this.authTickets = checkNotNull(authTicketService);

    // resolve where files to be downloaded will be stored
    this.downloadDir = checkNotNull(applicationDirectories).getWorkDirectory("downloads");
    log.info("Downloads directory: {}", downloadDir);
  }

  @Override
  public Download get(String fileName, String authTicket) throws IOException {
    log.info("Download: {}", fileName);

    if (!authTickets.redeemTicket(authTicket)) {
      throw new IllegalArgumentException("Invalid authentication ticket");
    }

    File file = new File(downloadDir, fileName);

    ensureWithinDownloads(file);

    if (!file.exists() && file.isFile()) {
      log.warn("File {} not found in download directory (or is not a file)", file);
      return null;
    }

    return new Download(file.length(), new FileInputStream(file));
  }

  @Override
  public String move(File source, String name) throws IOException {
    File target = new File(downloadDir, name);
    ensureWithinDownloads(target);
    Files.move(source.toPath(), target.toPath());
    log.debug("Moved {} to {}", source, target);
    return target.getAbsolutePath();
  }

  @Override
  public String uniqueName(String prefix) {
    return prefix + new SimpleDateFormat("yyyyMMdd-HHmmss").format(new Date()) + "-" + counter.incrementAndGet();
  }

  /**
   * Ensure we do not leak references outside of the downloads directory, only direct children can be served
   */
  private void ensureWithinDownloads(File file) {
    checkArgument(file.getParentFile().equals(downloadDir), "Reference outside of downloads dir: " + file);
  }
}
