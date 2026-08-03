
package org.globalart.astrum.ui;

import java.io.IOException;
import java.net.URL;

import org.springframework.beans.factory.annotation.Autowired;

import org.springframework.context.ApplicationContext;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.ResourcePatternResolver;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.20
 */
@Component
public class UiUtil
{
  private static final String RESOURCE_PREFIX = ResourcePatternResolver.CLASSPATH_ALL_URL_PREFIX + "/static/**/";

  private final ApplicationContext context;

  @Autowired
  public UiUtil(final ApplicationContext context) {
    this.context = checkNotNull(context);
  }

  /**
   * @param filename
   * @return the path to the requested file
   */
  public String getPathForFile(final String filename) {
    try {
      for (Resource resource : context.getResources(RESOURCE_PREFIX + filename)) {
        URL url = resource.getURL();
        String fullPath = url.getPath();
        if (fullPath.contains("/static")) {
          return fullPath.substring(fullPath.indexOf("/static"));
        }
        return url.getPath();
      }
      return null;
    }
    catch (IOException e) {
      throw new RuntimeException(e);
    }
  }
}
