package org.globalart.astrum.repository.docker;

import java.util.Collections;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

public final class DockerConfig
{
  public static final String DOCKER = "docker";

  public static final String DOCKER_PROXY = "dockerProxy";

  public static final String HTTP_PORT = "httpPort";

  public static final String HTTPS_PORT = "httpsPort";

  public static final String FORCE_BASIC_AUTH = "forceBasicAuth";

  public static final String V1_ENABLED = "v1Enabled";

  public static final String SUBDOMAIN = "subdomain";

  public static final String PATH_ENABLED = "pathEnabled";

  public static final String INDEX_TYPE = "indexType";

  public static final String INDEX_URL = "indexUrl";

  public static final String CACHE_FOREIGN_LAYERS = "cacheForeignLayers";

  public static final String FOREIGN_LAYER_URL_WHITELIST = "foreignLayerUrlWhitelist";

  public static final String INDEX_TYPE_REGISTRY = "REGISTRY";

  private DockerConfig() {
  }

  public static NestedAttributesMap docker(final Configuration configuration) {
    return configuration.attributes(DOCKER);
  }

  @Nullable
  public static Integer httpPort(final Configuration configuration) {
    return integer(docker(configuration).get(HTTP_PORT));
  }

  @Nullable
  public static Integer httpsPort(final Configuration configuration) {
    return integer(docker(configuration).get(HTTPS_PORT));
  }

  public static boolean forceBasicAuth(final Repository repository) {
    return bool(docker(repository.getConfiguration()).get(FORCE_BASIC_AUTH), false);
  }

  public static boolean pathEnabled(final Repository repository) {
    return bool(docker(repository.getConfiguration()).get(PATH_ENABLED), true);
  }

  @Nullable
  public static Integer integer(@Nullable final Object value) {
    if (value instanceof Number number) {
      return number.intValue();
    }
    if (value instanceof String text && !text.isBlank()) {
      try {
        return Integer.parseInt(text.trim());
      }
      catch (NumberFormatException e) {
        return null;
      }
    }
    return null;
  }

  public static boolean bool(@Nullable final Object value, final boolean defaultValue) {
    if (value instanceof Boolean flag) {
      return flag;
    }
    if (value instanceof String text && !text.isBlank()) {
      return Boolean.parseBoolean(text.trim());
    }
    return defaultValue;
  }

  @SuppressWarnings("unchecked")
  public static List<String> stringList(@Nullable final Object value) {
    if (value instanceof List<?> list) {
      return (List<String>) list;
    }
    return Collections.emptyList();
  }
}
