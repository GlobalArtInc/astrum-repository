
package org.globalart.astrum.repository.apt.datastore.internal;

import java.util.List;

import org.globalart.astrum.repository.apt.datastore.internal.browse.AptBrowseNodeGenerator;
import org.globalart.astrum.repository.content.browse.BrowseTestSupport;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.core.Is.is;

/**
 * @since 3.31
 */
public class AptBrowseNodeGeneratorTest
    extends BrowseTestSupport
{
  private AptBrowseNodeGenerator generator = new AptBrowseNodeGenerator();

  @Test
  public void computeComponentPath() {
    ComponentData componentData = new ComponentData();
    componentData.setRepositoryId(1);
    componentData.setComponentId(1);
    componentData.setName("nano");
    componentData.setNamespace("amd64");
    componentData.setVersion("1.0.0");

    AssetData asset = new AssetData();
    asset.setComponent(componentData);
    asset.setPath("/path/assetName");

    List<BrowsePath> paths = generator.computeComponentPaths(asset);

    assertThat(paths.size(), is(6));

    assertThat(paths, containsInAnyOrder(
        new BrowsePath("packages", "/packages/"),
        new BrowsePath("n", "/packages/n/"),
        new BrowsePath("nano", "/packages/n/nano/"),
        new BrowsePath("1.0.0", "/packages/n/nano/1.0.0/"),
        new BrowsePath("amd64", "/packages/n/nano/1.0.0/amd64/"),
        new BrowsePath("nano", "/packages/n/nano/1.0.0/amd64/nano/")));
  }

  @Test
  public void computeAssetPathMetadata() {
    AssetData asset = new AssetData();
    asset.setPath("/path/assetName");

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertThat(paths.size(), is(3));

    assertThat(paths, containsInAnyOrder(
        new BrowsePath("metadata", "/metadata/"),
        new BrowsePath("path", "/metadata/path/"),
        new BrowsePath("assetName", "/metadata/path/assetName/")));
  }

  @Test
  public void computeAssetPathDeb() {
    AssetData asset = new AssetData();
    asset.setPath("/path/assetName.deb");

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertThat(paths.size(), is(3));

    assertThat(paths, containsInAnyOrder(
        new BrowsePath("packages", "/packages/"),
        new BrowsePath("path", "/packages/path/"),
        new BrowsePath("assetName.deb", "/packages/path/assetName.deb/")));
  }

  @Test
  public void computeAssetPathSnapshots() {
    AssetData asset = new AssetData();
    asset.setPath("/snapshots/path/assetName");

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertThat(paths.size(), is(3));

    assertThat(paths, containsInAnyOrder(
        new BrowsePath("snapshots", "/snapshots/"),
        new BrowsePath("path", "/snapshots/path/"),
        new BrowsePath("assetName", "/snapshots/path/assetName/")));
  }
}
