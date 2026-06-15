
package org.globalart.astrum.coreui.internal.log;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Set;
import java.util.stream.Stream;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.logging.task.TaskLogHome;
import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.CONTENT_DISPOSITION;
import static java.lang.String.format;
import static java.util.stream.Collectors.toSet;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.TEXT_PLAIN;
import org.springframework.stereotype.Component;

/**
 * Logs REST resource.
 *
 * @since 3.3
 */
@Component
@Path(LogsResource.RESOURCE_URI)
public class LogsResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = APIConstants.INTERNAL_API_PREFIX + "/logging/logs";

  public static final String DEFAULT_MARK = "MARK";

  private final LogManager logManager;

  @Autowired
  public LogsResource(final LogManager logManager) {
    this.logManager = checkNotNull(logManager);
  }

  /**
   * List the log files known by the system
   */
  @GET
  @Produces({APPLICATION_JSON})
  @RequiresPermissions("astrum:logging:read")
  public Set<LogXO> listLogs() throws IOException {

    Set<LogXO> logs = logManager.getLogFiles().stream().map(file -> new LogXO(file.toPath())).collect(toSet());

    if (TaskLogHome.getTaskLogsHome() != null) {
      aggregateLogs(logs, TaskLogHome.getTaskLogsHome());
    }

    if (TaskLogHome.getReplicationLogsHome().isPresent()) {
      aggregateLogs(logs, TaskLogHome.getReplicationLogsHome().get());
    }

    return logs;
  }

  private void aggregateLogs(final Set<LogXO> logs, final String pathname) throws IOException {
    if (pathname != null) {
      try (Stream<java.nio.file.Path> paths = Files.list(Paths.get(pathname))) {
        paths.filter(logManager::isValidLogFile).forEach(path -> logs.add(new LogXO(path)));
      }
    }
  }

  /**
   * Downloads a part of a log file or the complete log file if fromByte/bytesCount are null.
   */
  @GET
  @Path("/{filename: .*\\.log}")
  @Produces({TEXT_PLAIN})
  @RequiresPermissions("astrum:logging:read")
  public Response get(
      @PathParam("filename") final String filename,
      @QueryParam("fromByte") final Long fromByte,
      @QueryParam("bytesCount") final Long bytesCount) throws NotFoundException, IOException
  {
    Long from = fromByte;
    if (from == null || from < 0) {
      from = 0L;
    }
    Long count = bytesCount;
    if (count == null) {
      count = Long.MAX_VALUE;
    }
    InputStream log = logManager.getLogFileStream(filename, from, count);
    if (log == null) {
      throw new NotFoundException(format("%s not found", filename));
    }
    return Response.ok(log)
        .header(CONTENT_DISPOSITION, format("attachment; filename=\"%s\"", filename))
        .build();
  }
}
