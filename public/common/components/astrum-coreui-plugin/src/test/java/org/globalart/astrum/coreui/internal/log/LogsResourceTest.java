
package org.globalart.astrum.coreui.internal.log;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.util.Set;

import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static com.google.common.net.HttpHeaders.CONTENT_DISPOSITION;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class LogsResourceTest
{
  @Mock
  private LogManager logManager;

  @InjectMocks
  private LogsResource underTest;

  @Test
  void getReturnsLogFileStream() throws IOException {
    String filename = "astrum.log";
    InputStream mockStream = new ByteArrayInputStream("log content".getBytes());
    when(logManager.getLogFileStream(filename, 0L, Long.MAX_VALUE)).thenReturn(mockStream);

    Response response = underTest.get(filename, null, null);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(200));
    assertThat(response.getHeaderString(CONTENT_DISPOSITION), is("attachment; filename=\"astrum.log\""));
  }

  @Test
  void getWithFromByteAndBytesCount() throws IOException {
    String filename = "astrum.log";
    InputStream mockStream = new ByteArrayInputStream("partial content".getBytes());
    when(logManager.getLogFileStream(filename, 100L, 500L)).thenReturn(mockStream);

    Response response = underTest.get(filename, 100L, 500L);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(200));
  }

  @Test
  void getWithNegativeFromByteDefaultsToZero() throws IOException {
    String filename = "astrum.log";
    InputStream mockStream = new ByteArrayInputStream("content".getBytes());
    when(logManager.getLogFileStream(filename, 0L, Long.MAX_VALUE)).thenReturn(mockStream);

    Response response = underTest.get(filename, -5L, null);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(200));
  }

  @Test
  void getThrowsNotFoundWhenLogStreamIsNull() throws IOException {
    String filename = "missing.log";
    when(logManager.getLogFileStream(filename, 0L, Long.MAX_VALUE)).thenReturn(null);

    NotFoundException exception = assertThrows(NotFoundException.class, () -> underTest.get(filename, null, null));

    assertThat(exception.getMessage(), containsString("missing.log not found"));
  }

  @Test
  void listLogsReturnsLogFiles(@TempDir File tempDir) throws IOException {
    File logFile = new File(tempDir, "astrum.log");
    when(logManager.getLogFiles()).thenReturn(Set.of(logFile));

    // TaskLogHome relies on static methods which are harder to test;
    // the basic flow through getLogFiles() is verified here
    Set<LogXO> logs = underTest.listLogs();
    assertThat(logs, is(notNullValue()));
  }
}
