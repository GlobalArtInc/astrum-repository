
package org.globalart.astrum.repository.p2.internal.browse;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.junit.Test;

import static java.lang.String.join;
import static java.util.Optional.empty;
import static java.util.Optional.of;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class P2BrowseNodeGeneratorTest
{
  private static final String[] KNOWN_SUB_DIRECTORIES = new String[]{"plugins", "features", "binary"};

  private static final String[] COMPONENTLESS_FILES = new String[]{
      "p2.index", "artifacts.xml", "artifacts.jar", "artifacts.xml.xz",
      "content.xml", "content.jar", "content.xml.xz", "compositeArtifacts.xml", "compositeArtifacts.jar",
      "compositeContent.xml", "compositeContent.jar"
  };

  private static final String SUBSITE = "945d37142fbf9b4c2885a99ebcf353ab852479348e7e86446557f7f05fe87e83";

  private static final List<String> COMPONENT_PATH = Arrays.asList("org", "tigris", "subversion", "clientadapter", "svnkit");

  private static final String COMPONENT_NAME = join(".", COMPONENT_PATH);

  private static final String COMPONENT_VERSION = "1.7.5";

  private static final String ASSET_NAME = join("_", COMPONENT_NAME, COMPONENT_VERSION);

  private P2BrowseNodeGenerator generator = new P2BrowseNodeGenerator();

  @Test
  public void testRepositoryAtRoot() {
    for (String file : COMPONENTLESS_FILES) {
      Asset asset = createAsset(file, false);

      List<String> paths = generator.computeAssetPaths(asset).stream().map(BrowsePath::getDisplayName)
          .collect(Collectors.toList());
      assertThat(paths, contains(file));
    }

    for (String directory : KNOWN_SUB_DIRECTORIES) {
      Asset asset = createAsset(join("/", directory, ASSET_NAME), true);

      List<String> paths = generator.computeAssetPaths(asset).stream()
          .map(BrowsePath::getDisplayName)
          .collect(Collectors.toList());

      List<String> expectedPaths = new ArrayList<>();
      expectedPaths.add(directory);
      expectedPaths.addAll(COMPONENT_PATH);
      expectedPaths.add(COMPONENT_VERSION);
      expectedPaths.add(ASSET_NAME);
      assertThat(paths, contains(expectedPaths.toArray()));
    }
  }

  @Test
  public void testNestedRepository() {
    for (String file : COMPONENTLESS_FILES) {
      String path = SUBSITE + "/" + file;
      Asset asset = createAsset(path, false);

      List<String> paths = generator.computeAssetPaths(asset).stream().map(BrowsePath::getDisplayName)
          .collect(Collectors.toList());
      assertThat(paths, contains(SUBSITE, file));
    }

    for (String directory : KNOWN_SUB_DIRECTORIES) {
      Asset asset = createAsset(join("/", SUBSITE, directory, ASSET_NAME), true);

      List<String> paths = generator.computeAssetPaths(asset).stream().map(BrowsePath::getDisplayName)
          .collect(Collectors.toList());

      List<String> expectedPaths = new ArrayList<>();
      expectedPaths.add(SUBSITE);
      expectedPaths.add(directory);
      expectedPaths.addAll(COMPONENT_PATH);
      expectedPaths.add(COMPONENT_VERSION);
      expectedPaths.add(ASSET_NAME);
      assertThat(paths, contains(expectedPaths.toArray()));
    }
  }

  @Test
  public void assetNodesUseTheAssetPathAsRequestPath() {
    for (String path : new String[]{"/p2.index", "/" + SUBSITE + "/p2.index",
        "/" + SUBSITE + "/plugins/" + ASSET_NAME + ".jar"}) {
      List<BrowsePath> browsePaths = generator.computeAssetPaths(createAsset(path, path.endsWith(".jar")));
      assertThat(browsePaths.get(browsePaths.size() - 1).getRequestPath(), is(path));
      for (BrowsePath browsePath : browsePaths.subList(0, browsePaths.size() - 1)) {
        assertThat(browsePath.getRequestPath().endsWith("/"), is(true));
      }
    }
  }

  private Asset createAsset(final String path, final boolean createComponent) {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn(path);

    if (createComponent) {
      Component component = mock(Component.class);
      when(component.namespace()).thenReturn(null);
      when(component.name()).thenReturn(COMPONENT_NAME);
      when(component.version()).thenReturn(COMPONENT_VERSION);

      when(asset.component()).thenReturn(of(component));
    } else {
      when(asset.component()).thenReturn(empty());
    }

    return asset;
  }
}
