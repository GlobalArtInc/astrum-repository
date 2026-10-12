package org.globalart.astrum.repository.huggingface.internal.browse;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class HuggingFaceBrowseNodeGeneratorTest
{
  private static final String COMMIT = "8f4dff0307bd726d0932f44183f1a88e297bcfe4";

  private final HuggingFaceBrowseNodeGenerator underTest = new HuggingFaceBrowseNodeGenerator();

  @Test
  void placesFilesUnderTheirCommit() {
    Asset asset = asset("/models/acme/tiny/" + COMMIT + "/onnx/model.onnx", component("models", "acme/tiny"));

    assertThat(names(underTest.computeAssetPaths(asset)), contains("models", "acme", "tiny", COMMIT, "onnx",
        "model.onnx"));
    List<BrowsePath> component = underTest.computeComponentPaths(asset);
    assertThat(names(component), contains("models", "acme", "tiny", COMMIT));
    assertThat(component.get(3).getRequestPath(), is("/models/acme/tiny/" + COMMIT + "/"));
  }

  @Test
  void handlesRepositoriesWithoutNamespace() {
    Asset asset = asset("/models/gpt2/" + COMMIT + "/config.json", component("models", "gpt2"));
    assertThat(names(underTest.computeComponentPaths(asset)), contains("models", "gpt2", COMMIT));
  }

  @Test
  void placesRevisionInformationNextToTheCommits() {
    Asset asset = asset("/datasets/acme/data/main.json", null);
    assertThat(names(underTest.computeAssetPaths(asset)), contains("datasets", "acme", "data", "main.json"));
  }

  private static Asset asset(final String path, final Component component) {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn(path);
    when(asset.component()).thenReturn(Optional.ofNullable(component));
    return asset;
  }

  private static Component component(final String namespace, final String name) {
    Component component = mock(Component.class);
    when(component.namespace()).thenReturn(namespace);
    when(component.name()).thenReturn(name);
    when(component.version()).thenReturn(COMMIT);
    return component;
  }

  private static List<String> names(final List<BrowsePath> paths) {
    return paths.stream().map(BrowsePath::getDisplayName).toList();
  }
}
