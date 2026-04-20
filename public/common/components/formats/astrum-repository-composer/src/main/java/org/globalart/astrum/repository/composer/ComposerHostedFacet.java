
package org.globalart.astrum.repository.composer;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import javax.annotation.Nullable;
import java.io.IOException;
import java.util.Optional;

/**
 * Interface defining the features supported by Composer repository hosted facets.
 */
@Facet.Exposed
public interface ComposerHostedFacet
    extends Facet
{
  FluentAsset upload(String vendor, String project, String version, String sourceType, String sourceUrl,
                     String sourceReference, Payload payload) throws IOException;

  Content getPackagesJson() throws IOException;

  Content getListJson(String filter) throws IOException;

  Content getProviderJson(String vendor, String project) throws IOException;

  Content getPackageJson(String vendor, String project) throws IOException;

  Optional<Content> rebuildPackageJson(String vendor, String project) throws IOException;

  Optional<Content> rebuildProviderJson(String vendor, String project) throws IOException;

  @Nullable
  Content getZipball(String path) throws IOException;
}
