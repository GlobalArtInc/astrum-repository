
package org.globalart.astrum.repository.conan.internal.content.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.conan.ConanCoords;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  private static final int RECIPE_DEPTH = 4;

  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    List<String> segments = segments(asset.path());
    if (segments.size() <= RECIPE_DEPTH) {
      return super.computeAssetPaths(asset);
    }
    boolean collapse = collapse(segments);
    List<BrowsePath> paths = new ArrayList<>();
    StringBuilder requestPath = new StringBuilder("/");
    for (int i = 0; i < segments.size(); i++) {
      requestPath.append(segments.get(i));
      if (i < segments.size() - 1) {
        requestPath.append('/');
      }
      if (collapse && (i == 2 || i == 3)) {
        continue;
      }
      paths.add(new BrowsePath(segments.get(i), requestPath.toString()));
    }
    return paths;
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);
    List<String> segments = segments(asset.path());
    if (segments.size() <= RECIPE_DEPTH) {
      return super.computeComponentPaths(asset);
    }
    List<BrowsePath> paths = computeAssetPaths(asset);
    int depth = collapse(segments) ? 2 : RECIPE_DEPTH;
    return new ArrayList<>(paths.subList(0, Math.min(depth, paths.size() - 1)));
  }

  private static boolean collapse(final List<String> segments) {
    return ConanCoords.NONE.equals(segments.get(2)) && ConanCoords.NONE.equals(segments.get(3));
  }

  private static List<String> segments(final String path) {
    return Splitter.on('/').omitEmptyStrings().splitToList(path);
  }
}
