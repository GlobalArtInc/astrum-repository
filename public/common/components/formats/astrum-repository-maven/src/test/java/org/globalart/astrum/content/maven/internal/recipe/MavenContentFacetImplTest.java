
package org.globalart.astrum.content.maven.internal.recipe;

import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.content.maven.store.Maven2ComponentData;
import org.globalart.astrum.content.maven.store.Maven2ComponentStore;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponentBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.maven.internal.Maven2MavenPathParser;
import org.globalart.astrum.repository.maven.internal.hosted.metadata.MetadataRebuilder;
import org.globalart.astrum.repository.maven.internal.validation.MavenMetadataContentValidator;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
public class MavenContentFacetImplTest
{
  private MavenContentFacetImpl underTest;

  @Mock
  private FormatStoreManager formatStoreManager;

  @Mock
  private MetadataRebuilder metadataRebuilder;

  @Mock
  private MavenMetadataContentValidator metadataValidator;

  @Mock
  private EventManager eventManager;

  @Mock
  private Repository repository;

  @Mock
  private FluentAssets fluentAssets;

  @Mock(answer = Answers.RETURNS_SELF)
  private FluentAssetBuilder fluentAssetBuilder;

  @Mock
  private FluentAsset fluentAsset;

  @Mock
  private Content content;

  @Mock
  private FluentComponents fluentComponents;

  @Mock(answer = Answers.RETURNS_SELF)
  private FluentComponentBuilder fluentComponentBuilder;

  @Mock
  private FluentComponent fluentComponent;

  @Mock
  private NestedAttributesMap componentAttributes;

  @Mock
  private Maven2ComponentStore maven2ComponentStore;

  @Mock
  private VersionNormalizerService versionNormalizerService;

  @Mock
  private TempBlob tempBlob;

  private MavenPathParser mavenPathParser = new Maven2MavenPathParser();

  @BeforeEach
  public void setup() throws Exception {
    underTest = spy(new MavenContentFacetImpl(
        formatStoreManager,
        mavenPathParser,
        metadataRebuilder,
        metadataValidator,
        eventManager,
        true));

    underTest.attach(repository);
    lenient().when(underTest.components()).thenReturn(fluentComponents);
    lenient().when(underTest.assets()).thenReturn(fluentAssets);
    lenient().doReturn(versionNormalizerService).when(underTest).versionNormalizerService();

    when(formatStoreManager.componentStore(anyString())).thenReturn(maven2ComponentStore);
    ContentFacetStores contentStores = new ContentFacetStores(mock(), "", formatStoreManager, "");
    lenient().when(underTest.stores()).thenReturn(contentStores);
  }

  @Test
  public void shouldReturnContentWhenAssetExists() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/artifact/1.0/artifact-1.0.jar");

    when(fluentAssets.path("/com/example/artifact/1.0/artifact-1.0.jar")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.of(fluentAsset));
    when(fluentAsset.download()).thenReturn(content);

    Optional<Content> result = underTest.get(mavenPath);

    assertTrue(result.isPresent());
    assertThat(result.get(), is(content));
  }

  @Test
  public void shouldReturnEmptyWhenAssetDoesNotExist() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/artifact/1.0/artifact-1.0.jar");

    when(fluentAssets.path("/com/example/artifact/1.0/artifact-1.0.jar")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.empty());

    Optional<Content> result = underTest.get(mavenPath);

    assertFalse(result.isPresent());
  }

  @Test
  public void shouldReturnTrueWhenAssetExists() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/artifact/1.0/artifact-1.0.jar");

    when(fluentAssets.path("/com/example/artifact/1.0/artifact-1.0.jar")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.of(fluentAsset));

    boolean result = underTest.exists(mavenPath);

    assertTrue(result);
  }

  @Test
  public void shouldReturnFalseWhenAssetDoesNotExist() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/artifact/1.0/artifact-1.0.jar");

    when(fluentAssets.path("/com/example/artifact/1.0/artifact-1.0.jar")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.empty());

    boolean result = underTest.exists(mavenPath);

    assertFalse(result);
  }

  @Test
  public void shouldDeleteAssetWhenItExists() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/maven-metadata.xml");

    when(fluentAssets.path("/com/example/maven-metadata.xml")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.of(fluentAsset));
    when(fluentAsset.delete()).thenReturn(true);

    boolean result = underTest.delete(mavenPath);

    assertTrue(result);
    verify(fluentAsset).delete();
  }

  @Test
  public void shouldReturnFalseWhenDeletingNonExistentAsset() {
    MavenPath mavenPath = mavenPathParser.parsePath("com/example/maven-metadata.xml");

    when(fluentAssets.path("/com/example/maven-metadata.xml")).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.empty());

    boolean result = underTest.delete(mavenPath);

    assertFalse(result);
  }

  @Test
  public void testCopyComponent() throws Exception {
    when(fluentComponents.name(anyString())).thenReturn(fluentComponentBuilder);
    when(underTest.contentRepositoryId()).thenReturn(123);

    Component source = mock();
    when(source.namespace()).thenReturn("tomcat");
    when(source.name()).thenReturn("catalina");
    when(source.version()).thenReturn("5.0.28");
    // maven doesn't use kind today future proof
    when(source.kind()).thenReturn("plugin");

    NestedAttributesMap attr = new NestedAttributesMap();
    when(source.attributes()).thenReturn(attr);

    FluentComponent created = mock();
    when(created.normalizedVersion()).thenReturn("005.000.028");
    when(created.attributes()).thenReturn(new NestedAttributesMap());
    when(fluentComponentBuilder.getOrCreate()).thenReturn(created);

    underTest.copy(source);

    verify(fluentComponents).name("catalina");
    verify(fluentComponentBuilder).namespace("tomcat");
    verify(fluentComponentBuilder).version("5.0.28");
    verify(fluentComponentBuilder).kind("plugin");

    ArgumentCaptor<Maven2ComponentData> captor = ArgumentCaptor.forClass(Maven2ComponentData.class);
    verify(maven2ComponentStore).updateBaseVersion(captor.capture());
    assertThat(captor.getValue().normalizedVersion(), is("005.000.028"));
  }
}
