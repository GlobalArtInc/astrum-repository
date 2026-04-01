
package org.globalart.astrum.repository.pypi.internal.content.browse;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.List;


import org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;

import com.google.common.base.Splitter;

import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;

/**
 * @since 3.29
 */
@Component
@Qualifier(PyPiFormat.NAME)
public class PypiBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    if (ContentPypiPathUtils.indexPath().equals(asset.path())) {
      List<String> pathSegments = Splitter.on(SLASH_CHAR).omitEmptyStrings().splitToList(asset.path());
      return BrowsePathBuilder.fromPaths(pathSegments, true);
    }
    return super.computeAssetPaths(asset);
  }
}
