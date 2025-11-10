
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Properties;
import java.util.Set;
import java.util.stream.Collectors;

import org.codehaus.plexus.interpolation.EnvarBasedValueSource;
import org.codehaus.plexus.interpolation.InterpolationException;
import org.codehaus.plexus.interpolation.Interpolator;
import org.codehaus.plexus.interpolation.MapBasedValueSource;
import org.codehaus.plexus.interpolation.StringSearchInterpolator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.context.annotation.Lazy;
import org.springframework.core.env.PropertySource;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.util.Collections.emptySet;
import static java.util.Collections.singleton;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.BASEDIR_SYS_PROP;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.DATADIR_SYS_PROP;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.getBasePath;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.getDataPath;

@Lazy
@Component
public class AstrumProperties
    extends PropertySource<String>
{
  private static final String INTERNAL_DEFAULT_PATH = "/org/globalart/astrum/bootstrap/application/default.properties";

  private static final File EXTERNAL_DEFAULT_ASTRUM_PROPERTIES_FILEPATH =
      getBasePath("etc", "astrum-default.properties").toFile();

  private static final File EXTERNAL_ASTRUM_PROPERTIES_FILEPATH = getDataPath("etc", "astrum.properties").toFile();

  private static final File EXTERNAL_NODENAME_FILEPATH = getDataPath("etc", "astrum-nodename.properties").toFile();

  private static final Logger LOG = LoggerFactory.getLogger(AstrumProperties.class);

  private static PropertyMap astrumProperties;

  private final AstrumPropertiesVerifier astrumPropertiesVerifier = new AstrumPropertiesVerifier();

  public AstrumProperties() {
    super("astrum-properties");
    init();
  }

  AstrumProperties(final PropertyMap propertiesForTesting) {
    super("astrum-properties");
    astrumProperties = propertiesForTesting;
  }

  @Override
  public String getProperty(final String propertyName) {
    try {
      Interpolator interpolator = new StringSearchInterpolator();
      interpolator.addValueSource(new MapBasedValueSource(astrumProperties));
      interpolator.addValueSource(new MapBasedValueSource(System.getProperties()));
      interpolator.addValueSource(new EnvarBasedValueSource());

      String propertyValue = astrumProperties.get(propertyName);

      if (propertyValue != null) {
        propertyValue = interpolator.interpolate(propertyValue);
      }
      return propertyValue;
    }
    catch (IOException | InterpolationException e) {
      throw new RuntimeException("Failed to interpolate astrum.properties entry: " + propertyName, e);
    }
  }

  void put(final String key, final String value) {
    astrumProperties.put(key, value);
  }

  String get(final String property, final String defaultValue) {
    String value = getProperty(property);

    return value != null ? value : defaultValue;
  }

  public Map<String, String> get() {
    return Map.copyOf(astrumProperties);
  }

  private void init() {
    if (astrumProperties != null) {
      return;
    }

    try {
      maybeCopyDefaults();

      astrumProperties = new PropertyMap();
      applyClasspathProperties(astrumProperties);
      applyProperties(astrumProperties, EXTERNAL_DEFAULT_ASTRUM_PROPERTIES_FILEPATH, true, emptySet());
      applyProperties(astrumProperties, EXTERNAL_ASTRUM_PROPERTIES_FILEPATH, false, emptySet());
      applyProperties(astrumProperties, EXTERNAL_NODENAME_FILEPATH, false, singleton("astrum.clustered.nodeName"));
      applySystemProperties(astrumProperties);

      interpolate();

      canonicalize(astrumProperties, BASEDIR_SYS_PROP);
      canonicalize(astrumProperties, "karaf.etc");
      canonicalize(astrumProperties, DATADIR_SYS_PROP);

      astrumPropertiesVerifier.verify(this);

      astrumProperties.forEach(System::setProperty);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private void applyClasspathProperties(final PropertyMap astrumProperties) throws IOException {
    URL resource = getClass().getResource(INTERNAL_DEFAULT_PATH);
    if (resource != null) {
      Properties properties = new Properties();
      try (InputStream inputStream = resource.openStream()) {
        properties.load(inputStream);
      }
      astrumProperties.putAll(properties);
      LOG.debug("astrum.properties loaded from classpath {}", INTERNAL_DEFAULT_PATH);
    }
  }

  private static void applyProperties(
      final PropertyMap astrumProperties,
      final File externalFilepath,
      final boolean required,
      final Set<String> keysToWatch) throws IOException
  {
    PropertyMap props = new PropertyMap();
    try {
      props.load(externalFilepath.toURI().toURL());
      if (keysToWatch == null || keysToWatch.isEmpty()) {
        astrumProperties.putAll(props);
        LOG.debug("astrum.properties loaded from file {}", externalFilepath.getAbsolutePath());
      }
      else {
        for (Entry<String, String> entry : props.entrySet()) {
          String key = entry.getKey();
          String value = entry.getValue();
          if (keysToWatch.contains(key)) {
            astrumProperties.put(key, value);
            LOG.debug("Overriding key {} from file {}", key, externalFilepath);
          }
        }
      }
    }
    catch (IOException e) {
      if (required) {
        throw e;
      }

      LOG.debug("Failed to load properties from non-required file: {}", externalFilepath);
    }
  }

  private static void applySystemProperties(final PropertyMap astrumProperties) {
    for (Entry<Object, Object> entry : System.getProperties().entrySet()) {
      String key = entry.getKey().toString();
      astrumProperties.put(key, entry.getValue().toString());
      LOG.debug("Overriding key {} from System.properties", key);
    }
  }

  private static void maybeCopyDefaults() throws IOException {
    if (EXTERNAL_DEFAULT_ASTRUM_PROPERTIES_FILEPATH.exists() && !EXTERNAL_ASTRUM_PROPERTIES_FILEPATH.exists()) {
      File parentDir = EXTERNAL_ASTRUM_PROPERTIES_FILEPATH.getParentFile();
      if (parentDir != null && !parentDir.isDirectory()) {
        Files.createDirectories(parentDir.toPath());
      }

      // Get list of default properties, commented out
      List<String> defaultProperties =
          getDefaultPropertiesCommentedOut(EXTERNAL_DEFAULT_ASTRUM_PROPERTIES_FILEPATH.toPath());

      Files.write(EXTERNAL_ASTRUM_PROPERTIES_FILEPATH.toPath(), defaultProperties, ISO_8859_1);
    }
  }

  private static List<String> getDefaultPropertiesCommentedOut(final Path defaultPropertiesPath) throws IOException {
    return Files.readAllLines(defaultPropertiesPath, ISO_8859_1)
        .stream()
        .filter(l -> !l.startsWith("##"))
        .map(l -> l.startsWith("#") || l.isEmpty() ? l : "# " + l)
        .collect(Collectors.toList());
  }

  private static void canonicalize(final PropertyMap nexuProperties, final String name) throws IOException {
    String value = nexuProperties.get(name);
    if (value == null) {
      LOG.warn("Unable to canonicalize null entry: {}", name);
      return;
    }
    File file = new File(value).getCanonicalFile();
    nexuProperties.put(name, file.getPath());
  }

  private void interpolate() throws IOException {
    Interpolator interpolator = new StringSearchInterpolator();
    interpolator.addValueSource(new MapBasedValueSource(astrumProperties));
    interpolator.addValueSource(new MapBasedValueSource(System.getProperties()));
    interpolator.addValueSource(new EnvarBasedValueSource());

    for (Entry<String, String> entry : astrumProperties.entrySet()) {
      try {
        astrumProperties.put(entry.getKey(), interpolator.interpolate(entry.getValue()));
      }
      catch (InterpolationException e) {
        throw new RuntimeException(
            "Failed to interpolate astrum.properties entry: " + entry.getKey() + "/" + entry.getValue(), e);
      }
    }
  }
}
