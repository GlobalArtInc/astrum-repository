
package org.globalart.astrum.repository.p2.internal.browse;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;

import com.google.common.base.Splitter;

import static com.google.common.collect.ImmutableSet.of;

public class P2BrowseNodeGeneratorHelper
{
  private static final Set<String> knownSubDirectories = of("binary", "features", "plugins");

  private P2BrowseNodeGeneratorHelper() {}

  public static List<String> computeComponentPath(final List<String> assetPath, final Optional<String> nameOptional, final Optional<String> versionOptional) {
    List<String> pathParts = new ArrayList<>();
    if (assetPath.size() > 1) {
      pathParts.add(assetPath.get(0));
    }

    nameOptional.ifPresent(name -> {
      String version = versionOptional.orElseThrow(() -> new IllegalStateException("Supplied component name, but not version"));
      if (!knownSubDirectories.contains(assetPath.get(0))) {
        pathParts.add(assetPath.get(1));
      }

      pathParts.addAll(Splitter.on('.').omitEmptyStrings().splitToList(name));
      pathParts.add(version);
    });

    return pathParts;
  }

  public static List<String> splitPath(final String path) {
    return Splitter.on('/').omitEmptyStrings().splitToList(path);
  }
}
