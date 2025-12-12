
package org.globalart.astrum.internal.atlas.customizers;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileWriter;
import java.io.InputStreamReader;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class LogCustomizerTest
{
  @TempDir
  File temporaryWorkDirectory;

  @Mock
  private LogManager mockLogManager;

  @Mock
  private ApplicationDirectories mockApplicationDirectories;

  private LogCustomizer underTest;

  @BeforeEach
  void setup() {
    when(mockApplicationDirectories.getWorkDirectory()).thenReturn(temporaryWorkDirectory);
    underTest = new LogCustomizer(mockLogManager, mockApplicationDirectories);
  }

  @Test
  void customizeAstrumLogRedactsAuthorizationBearer() throws Exception {
    String logContent =
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth request - Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9\n";
    testLogRedaction(logContent,
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth request - Authorization: Bearer ****");
  }

  @Test
  void customizeAstrumLogRedactsAuthorizationBasic() throws Exception {
    String logContent =
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: Basic dXNlcjpwYXNzd29yZA==\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: Basic ****");
  }

  @Test
  void customizeAstrumLogRedactsAuthorizationToken() throws Exception {
    String logContent =
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: Token token=\"abc123\"\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: Token ****");
  }

  @Test
  void customizeAstrumLogRedactsWWWAuthenticate() throws Exception {
    String logContent =
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] response - WWW-Authenticate: Bearer realm=\"example\"\n";
    testLogRedaction(logContent,
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] response - WWW-Authenticate: Bearer ****");
  }

  @Test
  void customizeAstrumLogRedactsProxyAuthorization() throws Exception {
    String logContent =
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] proxy - Proxy-Authorization: Basic dXNlcjpwYXNz\n";
    testLogRedaction(logContent,
        "2024-01-15 10:30:45,123 INFO  [executor-thread-1] proxy - Proxy-Authorization: Basic ****");
  }

  @Test
  void customizeAstrumLogRedactsXAuthToken() throws Exception {
    String logContent = "2024-01-15 10:30:45,123 INFO  [executor-thread-1] api - X-Auth-Token: abc123def456\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] api - X-Auth-Token: ****");
  }

  @Test
  void customizeAstrumLogRedactsXApiKey() throws Exception {
    String logContent = "2024-01-15 10:30:45,123 INFO  [executor-thread-1] api - X-API-Key: secret-key-123\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] api - X-API-Key: ****");
  }

  @Test
  void customizeAstrumLogRedactsAuthorizationWithoutScheme() throws Exception {
    String logContent = "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: rawtoken123\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - Authorization: ****");
  }

  @Test
  void customizeAstrumLogRedactsCaseInsensitive() throws Exception {
    String logContent = "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - authorization: bearer token123\n";
    testLogRedaction(logContent, "2024-01-15 10:30:45,123 INFO  [executor-thread-1] auth - authorization: bearer ****");
  }

  @Test
  void customizeAstrumLogPreservesNonSensitiveContent() throws Exception {
    String logContent = "2024-01-15 10:30:45,123 INFO  [executor-thread-1] request - GET /repository/maven-central/\n";
    testLogRedaction(logContent, logContent.trim());
  }

  @Test
  void customizeAstrumLogRedactsMultipleHeadersOnSameLine() throws Exception {
    String logContent = "Authorization: Bearer token123 and X-API-Key: secret456\n";
    // With .+, the first match consumes to end of line, so "and X-API-Key: secret456" is also redacted
    testLogRedaction(logContent, "Authorization: Bearer ****");
  }

  private void testLogRedaction(final String inputContent, final String expectedOutput) throws Exception {
    File logFile = new File(temporaryWorkDirectory, "log/astrum.log");
    logFile.getParentFile().mkdirs();
    logFile.createNewFile();

    try (FileWriter writer = new FileWriter(logFile)) {
      writer.write(inputContent);
    }

    when(mockLogManager.getLogFor(anyString())).thenReturn(Optional.of("astrum.log"));
    when(mockLogManager.getLogFileStream(anyString(), anyLong(), anyLong()))
        .thenReturn(new java.io.FileInputStream(logFile));

    SupportBundle supportBundle = new SupportBundle();
    underTest.customize(supportBundle);

    List<ContentSource> list = supportBundle.getSources();
    assertThat(list.size(), equalTo(1));

    ContentSource contentSource = list.get(0);
    contentSource.prepare();

    try (BufferedReader reader = new BufferedReader(new InputStreamReader(contentSource.getContent()))) {
      assertThat(reader.readLine(), equalTo(expectedOutput));
    }
  }
}
