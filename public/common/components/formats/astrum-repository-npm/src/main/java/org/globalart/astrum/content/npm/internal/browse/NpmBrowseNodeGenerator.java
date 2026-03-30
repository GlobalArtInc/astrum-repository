package org.globalart.astrum.content.npm.internal.browse;

import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.npm.internal.NpmFormat;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;

@Component
@Qualifier(NpmFormat.NAME)
public class NpmBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    List<String> segments = Splitter.on(SLASH_CHAR).omitEmptyStrings().splitToList(asset.path()).stream()
        .filter(segment -> !"-".equals(segment))
        .collect(toList());
    return BrowsePathBuilder.fromPaths(segments, false);
  }
}
