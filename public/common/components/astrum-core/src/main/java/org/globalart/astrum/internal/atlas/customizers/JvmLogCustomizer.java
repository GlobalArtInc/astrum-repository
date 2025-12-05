
package org.globalart.astrum.internal.atlas.customizers;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.lang.management.ManagementFactory;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.text.Strings2.MASK;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.LOW;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.LOG;
import org.springframework.stereotype.Component;

/**
 * Adds jvm log file to support bundle.
 * Masks sensitive data passed as JVM arguments.
 */
@Component
public class JvmLogCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final List<String> SENSITIVE_FIELD_NAMES =
      Arrays.asList("password", "secret", "token", "sign", "auth", "cred", "key", "pass");

  private final LogManager logManager;

  private final ApplicationDirectories applicationDirectories;

  @Autowired
  public JvmLogCustomizer(final LogManager logManager, final ApplicationDirectories applicationDirectories) {
    this.logManager = checkNotNull(logManager);
    this.applicationDirectories = checkNotNull(applicationDirectories);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    supportBundle.add(new GeneratedContentSourceSupport(LOG, "log/jvm.log", LOW)
    {
      @Override
      protected void generate(final File file) {
        File logFile = resolveJvmLogFile();

        if (logFile.exists()) {
          try (BufferedReader reader = new BufferedReader(new FileReader(logFile));
              BufferedWriter writer = new BufferedWriter(new FileWriter(file))) {

            String line;
            while ((line = reader.readLine()) != null) {
              String redactedLine = maybeMaskSensitiveData(line);
              writer.write(redactedLine);
              writer.newLine();
            }
          }
          catch (IOException e) {
            log.debug("Unable to include jvm.log file", e);
          }
        }
        else {
          log.debug("Not including missing jvm.log file");
        }
      }

      private String maybeMaskSensitiveData(final String input) {
        String result = input;
        for (String name : SENSITIVE_FIELD_NAMES) {
          result = result.replaceAll(name + "=\\S*", name + "=" + MASK);
        }
        return result;
      }
    });
  }

  private File resolveJvmLogFile() {
    File logFile = logManager.getLogFile("jvm.log");
    if (logFile != null) {
      log.debug("Resolved jvm.log via LogManager: {}", logFile);
      return logFile;
    }
    Optional<File> runtimeLog = resolveFromRuntimeArgs();
    if (runtimeLog.isPresent()) {
      log.debug("Couldn't find jvm.log via LogManager, resolved via runtime args: {}", runtimeLog.get());
      return runtimeLog.get();
    }
    log.debug("Couldn't find jvm.log via LogManager or runtime args, falling back to default location");
    return new File(applicationDirectories.getWorkDirectory(), "log/jvm.log");
  }

  private Optional<File> resolveFromRuntimeArgs() {
    List<String> args = ManagementFactory.getRuntimeMXBean().getInputArguments();
    log.debug("Runtime JVM args: {}", args);
    for (String arg : args) {
      String trimmed = arg.trim();
      if (trimmed.startsWith("-XX:LogFile=")) {
        String value = trimmed.substring("-XX:LogFile=".length()).trim();
        if (!value.isEmpty()) {
          return Optional.of(resolveLogPath(value));
        }
      }
    }
    return Optional.empty();
  }

  private File resolveLogPath(final String value) {
    // Expand common karaf placeholders and resolve relative paths against the install directory.
    String resolved = value
        .replace("${karaf.data}", applicationDirectories.getWorkDirectory().getAbsolutePath())
        .replace("${karaf.base}", applicationDirectories.getInstallDirectory().getAbsolutePath());
    File file = new File(resolved);
    if (file.isAbsolute()) {
      return file;
    }
    return new File(applicationDirectories.getInstallDirectory(), resolved);
  }

}
