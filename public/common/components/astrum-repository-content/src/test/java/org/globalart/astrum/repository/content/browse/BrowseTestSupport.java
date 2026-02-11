
package org.globalart.astrum.repository.content.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;

import static java.util.Optional.of;
import static java.util.Optional.ofNullable;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class BrowseTestSupport
{
  protected void assertPaths(List<String> expectedPaths, List<BrowsePath> paths) {
    assertPaths(expectedPaths, paths, false);
  }

  protected void assertPaths(List<String> expectedPaths, List<BrowsePath> paths, boolean withTrailingSlash) {
    assertPaths(expectedPaths, expectedPaths, paths, withTrailingSlash);
  }

  protected void assertPaths(
      List<String> expectedBrowsePaths,
      List<String> expectedRequestPaths,
      List<BrowsePath> paths)
  {
    assertPaths(expectedBrowsePaths, expectedRequestPaths, paths, false);
  }

  protected void assertPaths(
      List<String> expectedBrowsePaths,
      List<String> expectedRequestPaths,
      List<BrowsePath> paths,
      boolean withTrailingSlash)
  {
    assertThat(expectedRequestPaths.size(), is(expectedBrowsePaths.size()));
    assertThat(paths.size(), is(expectedBrowsePaths.size()));

    String requestPath = "/";

    for (int i = 0; i < expectedBrowsePaths.size(); i++) {
      requestPath += expectedRequestPaths.get(i);
      if (withTrailingSlash || i < expectedBrowsePaths.size() - 1) {
        requestPath += "/";
      }
      assertThat(paths.get(i).getDisplayName(), is(expectedBrowsePaths.get(i)));
      assertThat(paths.get(i).getRequestPath(), is(requestPath));
    }
  }

  protected Asset createAsset(final String path) {
    return createAsset(path, null);
  }

  protected Asset createAsset(final String path, final Component component) {
    Asset asset = mock(Asset.class);

    when(asset.blob()).thenReturn(of(mock(AssetBlob.class)));
    when(asset.component()).thenReturn(ofNullable(component));
    when(asset.path()).thenReturn(path);

    return asset;
  }

  protected Component createComponent(final String name, final String namespace, final String version) {
    Component component = mock(Component.class);

    when(component.name()).thenReturn(name);
    when(component.namespace()).thenReturn(namespace);
    when(component.version()).thenReturn(version);

    return component;
  }

  protected List<BrowsePath> toBrowsePaths(List<String> paths) {
    List<BrowsePath> results = new ArrayList<>();

    final StringBuilder requestPath = new StringBuilder();

    paths.forEach(path -> {
      requestPath.append(path);
      results.add(new BrowsePath(path, requestPath.toString()));
      requestPath.append("/");
    });

    return results;
  }
}
