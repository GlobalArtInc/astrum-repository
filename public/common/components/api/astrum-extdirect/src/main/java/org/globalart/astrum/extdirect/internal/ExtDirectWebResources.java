
package org.globalart.astrum.extdirect.internal;

import java.io.File;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.webresources.FileWebResource;
import org.globalart.astrum.webresources.WebResource;
import org.globalart.astrum.webresources.WebResourceBundle;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.webresources.WebResource.JAVASCRIPT;
import org.springframework.stereotype.Component;

/**
 * Ext.Direct web-resources.
 *
 * @since 3.0
 */
@Component
public class ExtDirectWebResources
    implements WebResourceBundle
{
  private final ApplicationDirectories directories;

  @Autowired
  public ExtDirectWebResources(final ApplicationDirectories directories) {
    this.directories = checkNotNull(directories);
  }

  private WebResource create(final String fileName, final String path) {
    File file = new File(directories.getTemporaryDirectory(), fileName);
    return new FileWebResource(file, path, JAVASCRIPT, true);
  }

  // FIXME: Would like to replace the generation here instead of relying on file which could be changed, etc
  // FIXME: Also we need a bit more control over the generation of this content so we can set the baseUrl etc

  @Override
  public List<WebResource> getResources() {
    return ImmutableList.of(
        create("astrum-extdirect/api.js", "/static/rapture/extdirect-prod.js"),
        create("astrum-extdirect/api-debug.js", "/static/rapture/extdirect-debug.js"));
  }
}
