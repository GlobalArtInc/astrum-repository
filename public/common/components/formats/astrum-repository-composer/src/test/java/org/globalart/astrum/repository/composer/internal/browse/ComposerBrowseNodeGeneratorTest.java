
package org.globalart.astrum.repository.composer.internal.browse;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;

import java.util.Arrays;
import java.util.List;

import static org.junit.Assert.assertEquals;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerBrowseNodeGeneratorTest
{
  private final ComposerBrowseNodeGenerator underTest = new ComposerBrowseNodeGenerator();

  @Test
  public void testComputeAssetPaths() {
    AssetData asset = new AssetData();
    asset.setPath("/v3nd0r/p4ck4g3/13.3.7/v3nd0r-p4ck4g3-13.3.7.zip");
    asset.setComponent(createComponent("v3nd0r", "p4ck4g3", "13.3.7"));
    List<BrowsePath> browsePaths = underTest.computeAssetPaths(asset);
    assertEquals(
        Arrays.asList(
            new BrowsePath("v3nd0r", "/v3nd0r/"),
            new BrowsePath("p4ck4g3", "/v3nd0r/p4ck4g3/"),
            new BrowsePath("13.3.7", "/v3nd0r/p4ck4g3/13.3.7/"),
            new BrowsePath("v3nd0r-p4ck4g3-13.3.7.zip", "/v3nd0r/p4ck4g3/13.3.7/v3nd0r-p4ck4g3-13.3.7.zip")
        ),
        browsePaths
    );
  }
  @Test
  public void computeComponentPaths() {
    AssetData asset = new AssetData();
    asset.setPath("/v3nd0r/p4ck4g3/13.3.7/v3nd0r-p4ck4g3-13.3.7.zip");
    asset.setComponent(createComponent("v3nd0r", "p4ck4g3", "13.3.7"));
    List<BrowsePath> browsePaths = underTest.computeAssetPaths(asset);
    assertEquals(
        Arrays.asList(
            new BrowsePath("v3nd0r", "/v3nd0r/"),
            new BrowsePath("p4ck4g3", "/v3nd0r/p4ck4g3/"),
            new BrowsePath("13.3.7", "/v3nd0r/p4ck4g3/13.3.7/"),
            new BrowsePath("v3nd0r-p4ck4g3-13.3.7.zip", "/v3nd0r/p4ck4g3/13.3.7/v3nd0r-p4ck4g3-13.3.7.zip")
        ),
        browsePaths
    );
  }

  private static Component createComponent(String vendor, String packageName, String version) {
    ComponentData componentData = new ComponentData();
    componentData.setRepositoryId(1);
    componentData.setComponentId(1);
    componentData.setNamespace(vendor);
    componentData.setName(packageName);
    componentData.setVersion(version);
    return componentData;
  }
}
