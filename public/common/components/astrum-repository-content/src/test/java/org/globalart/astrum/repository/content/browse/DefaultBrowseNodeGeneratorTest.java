
package org.globalart.astrum.repository.content.browse;

import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.junit.Before;
import org.junit.Test;

import static java.util.Arrays.asList;
import static java.util.Collections.singletonList;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class DefaultBrowseNodeGeneratorTest
    extends BrowseTestSupport
{
  private DefaultBrowseNodeGenerator generator;

  @Before
  public void setup() {
    generator = new DefaultBrowseNodeGenerator();
  }

  @Test
  public void computeAssetPathWithNoComponent() {
    Asset asset = createAsset("/path/asset");

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertPaths(asList("path", "asset"), paths);
  }

  @Test
  public void computeAssetPathWithNoComponent_trailingSlash() {
    Asset asset = createAsset("/path/asset/");

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertPaths(asList("path", "asset"), paths);
  }

  @Test
  public void computeAssetPathWithComponentNameOnly() {
    Component component = createComponent("component", null, null);
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertPaths(asList(component.name(), "assetName"), paths);
  }

  @Test
  public void computeAssetPathWithComponentNoGroup() {
    Component component = createComponent("component", null, "1.0.0");
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertPaths(asList(component.name(), component.version(), "assetName"), paths);
  }

  @Test
  public void computeAssetPathWithComponent() {
    Component component = createComponent("component", "group", "1.0.0");
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeAssetPaths(asset);

    assertPaths(asList(component.namespace(), component.name(), component.version(), "assetName"), paths);
  }

  @Test
  public void computeComponentPathWithComponentNameOnly() {
    Component component = createComponent("component", null, null);
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeComponentPaths(asset);

    assertPaths(singletonList(component.name()), paths, true);
  }

  @Test
  public void computeComponentPathWithComponentNoGroup() {
    Component component = createComponent("component", null, "1.0.0");
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeComponentPaths(asset);

    assertPaths(asList(component.name(), component.version()), paths, true);
  }

  @Test
  public void computeComponentPathWithComponent() {
    Component component = createComponent("component", "group", "1.0.0");
    Asset asset = createAsset("path/assetName", component);

    List<BrowsePath> paths = generator.computeComponentPaths(asset);

    assertPaths(asList(component.namespace(), component.name(), component.version()), paths, true);
  }

  @Test
  public void lastSegmentBehaviour() {
    assertThat(generator.lastSegment(""), is(""));
    assertThat(generator.lastSegment("/"), is(""));
    assertThat(generator.lastSegment("//"), is(""));
    assertThat(generator.lastSegment("///"), is(""));
    assertThat(generator.lastSegment("////"), is(""));
    assertThat(generator.lastSegment("a"), is("a"));
    assertThat(generator.lastSegment("a/"), is("a"));
    assertThat(generator.lastSegment("/a"), is("a"));
    assertThat(generator.lastSegment("/a/"), is("a"));
    assertThat(generator.lastSegment("//a"), is("a"));
    assertThat(generator.lastSegment("a//"), is("a"));
    assertThat(generator.lastSegment("//a//"), is("a"));
    assertThat(generator.lastSegment("a/b"), is("b"));
    assertThat(generator.lastSegment("a/b/"), is("b"));
    assertThat(generator.lastSegment("/a/b"), is("b"));
    assertThat(generator.lastSegment("/a/b/"), is("b"));
    assertThat(generator.lastSegment("a/.b"), is(".b"));
    assertThat(generator.lastSegment("a/b.c"), is("b.c"));
  }
}
