
package org.globalart.astrum.bootstrap.entrypoint.edition;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.nio.charset.StandardCharsets.UTF_8;

public abstract class AstrumEditionSupport
    implements AstrumEdition
{
  private static final Logger LOG = LoggerFactory.getLogger(AstrumEdition.class);

  private final String id;

  private final String name;

  private final String shortName;

  private final int priority;

  public AstrumEditionSupport(
      final String id,
      final String name,
      final String shortName,
      final int priority)
  {
    this.id = id;
    this.name = name;
    this.shortName = shortName;
    this.priority = priority;
  }

  @Override
  public String getId() {
    return id;
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public String getShortName() {
    return shortName;
  }

  @Override
  public int getPriority() {
    return priority;
  }

  @Override
  public List<String> getModules() {
    List<String> modules = new ArrayList<>();
    try (InputStream inputStream = getClass().getClassLoader().getResourceAsStream("BOOT-INF/classpath.idx")) {
      if (inputStream != null) {
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(inputStream, UTF_8))) {
          String line;
          while ((line = reader.readLine()) != null) {
            line = line.replace("\"", "").replace("- BOOT-INF/lib/", "");
            modules.add(line);
          }
        }
      }
    }
    catch (IOException e) {
      String msg = "Failed to load the classpath.idx file";
      LOG.error(msg, e);
      throw new RuntimeException(msg, e);
    }

    return modules;
  }
}
