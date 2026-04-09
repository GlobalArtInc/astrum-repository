package org.globalart.astrum.repository.helm.internal.util;

import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;
import org.yaml.snakeyaml.LoaderOptions;

public final class HelmYaml
{
  private static final int CODE_POINT_LIMIT = 512 * 1024 * 1024;

  private HelmYaml() {
  }

  public static YAMLFactory factory() {
    LoaderOptions options = new LoaderOptions();
    options.setCodePointLimit(CODE_POINT_LIMIT);
    return YAMLFactory.builder().loaderOptions(options).build();
  }
}
