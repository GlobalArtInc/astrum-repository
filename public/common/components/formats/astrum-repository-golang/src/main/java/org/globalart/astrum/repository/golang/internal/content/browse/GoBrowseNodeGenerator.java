package org.globalart.astrum.repository.golang.internal.content.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.golang.GolangFormat;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(GolangFormat.NAME)
public class GoBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  private static final String VERSIONS = "/@v/";

  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    if (asset.component().isPresent() && asset.path().contains(VERSIONS)) {
      List<BrowsePath> paths = computeComponentPaths(asset);
      BrowsePathBuilder.appendPath(paths, lastSegment(asset.path()), asset.path());
      return paths;
    }
    return BrowsePathBuilder.fromPaths(segments(asset.path()), false);
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);
    String path = asset.path();
    int versions = path.lastIndexOf(VERSIONS);
    if (versions < 0) {
      return super.computeComponentPaths(asset);
    }
    String file = path.substring(versions + VERSIONS.length());
    int dot = file.lastIndexOf('.');
    List<String> segments = new ArrayList<>(segments(path.substring(0, versions)));
    segments.add(dot > 0 ? file.substring(0, dot) : file);
    return BrowsePathBuilder.fromPaths(segments, true);
  }

  private static List<String> segments(final String path) {
    return Splitter.on('/').omitEmptyStrings().splitToList(path);
  }
}
