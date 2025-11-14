
package org.globalart.astrum.internal.webresources;

import java.io.File;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import org.globalart.astrum.mime.MimeSupport;
import org.globalart.astrum.webresources.FileWebResource;
import org.globalart.astrum.webresources.WebResource;
import org.globalart.astrum.webresources.WebResourceBundle;
import org.globalart.astrum.webresources.WebResourceService;

import com.google.common.collect.Maps;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link WebResourceService} implementation.
 *
 * @since 2.8
 */
@Component
public class WebResourceServiceImpl
    implements WebResourceService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DevModeResources devModeResources;

  private final MimeSupport mimeSupport;

  private final Map<String, WebResource> resourcePaths;

  @Autowired
  public WebResourceServiceImpl(
      final DevModeResources devModeResources,
      final MimeSupport mimeSupport,
      final List<WebResourceBundle> bundles,
      final List<WebResource> resources)
  {
    this.devModeResources = checkNotNull(devModeResources);
    this.mimeSupport = checkNotNull(mimeSupport);
    this.resourcePaths = Maps.newHashMap();

    bundles.stream()
        .map(WebResourceBundle::getResources)
        .flatMap(Collection::stream)
        .filter(Objects::nonNull)
        .forEach(this::addResource);

    resources.forEach(this::addResource);

    // make it clear we have DEV mode enabled
    List<File> locations = devModeResources.getResourceLocations();
    if (locations != null) {
      log.warn("DEV mode resources is ENABLED");
      // spit out the locations where we will look for resources
      for (File file : locations) {
        log.info("  {}", file);
      }
    }
  }

  void addResource(final WebResource resource) {
    String path = resource.getPath();
    log.trace("Adding resource: {} -> {}", path, resource);
    final WebResource old = resourcePaths.put(path, resource);
    if (old != null) {
      // complain if any resources overlap
      log.debug("Overlapping resources on path {}: old={}, new={}", path, old, resource);
    }
  }

  @Override
  public Collection<String> getPaths() {
    return Collections.unmodifiableCollection(resourcePaths.keySet());
  }

  @Override
  public Collection<WebResource> getResources() {
    return Collections.unmodifiableCollection(resourcePaths.values());
  }

  @Override
  public WebResource getResource(final String path) {
    log.trace("Looking up resource: {}", path);

    WebResource resource = null;

    // 1) first "dev" resources if enabled (to override everything else)
    File file = devModeResources.getFileIfOnFileSystem(path);
    if (file != null) {
      resource = new FileWebResource(file, path, mimeSupport.guessMimeTypeFromPath(file.getName()), false);
      log.trace("Found dev-mode resource: {}", resource);
    }

    // 2) second, look at "ordinary" resources, but only if devResource did not hit anything
    if (resource == null) {
      resource = resourcePaths.get(path);
      if (resource != null) {
        log.trace("Found bound resource: {}", resource);
      }
    }

    return resource;
  }
}
