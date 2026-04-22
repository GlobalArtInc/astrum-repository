
package org.globalart.astrum.repository.composer.internal;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.view.Content;

import java.io.IOException;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;

import static java.util.Collections.singleton;
import static org.junit.Assert.assertEquals;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerMaintenanceFacetTest
{
  private static final String REPOSITORY_NAME = "test-repository-name";

  private static final String VENDOR = "test-vendor";

  private static final String PROJECT = "test-project";
  private static final String ZIPBALL_PATH = "/" + VENDOR + "/" + PROJECT + "/1.0.0/" + VENDOR + "-" + PROJECT + "-1.0.0.zip";
  private static final String PROVIDER_PATH = "/p/" + VENDOR + "/" + PROJECT + ".json";
  private static final String PACKAGE_PATH = "/p2/" + VENDOR + "/" + PROJECT + ".json";


  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private ComposerHostedFacet hostedFacet;

  @Mock
  private Component component;

  @Mock
  private FluentComponents fluentComponents;

  @Mock
  private FluentComponent fluentComponent;

  @Mock
  private FluentAsset fluentAsset;

  @Mock
  private Content content;

  private ComposerMaintenanceFacet underTest;

  @Before
  public void setUp() throws Exception {
    when(repository.getName()).thenReturn(REPOSITORY_NAME);
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.optionalFacet(ComposerHostedFacet.class)).thenReturn(Optional.of(hostedFacet));

    when(contentFacet.components()).thenReturn(fluentComponents);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);
    when(fluentComponent.assets()).thenReturn(singleton(fluentAsset));
    when(fluentAsset.delete()).thenReturn(true);
    when(fluentAsset.path()).thenReturn(ZIPBALL_PATH);

    when(component.namespace()).thenReturn(VENDOR);
    when(component.name()).thenReturn(PROJECT);

    underTest = new ComposerMaintenanceFacet();
    underTest.attach(repository);
  }

  @Test
  public void testDeleteComponent() throws IOException {
    when(hostedFacet.rebuildPackageJson(VENDOR, PROJECT)).thenReturn(Optional.of(content));
    when(hostedFacet.rebuildProviderJson(VENDOR, PROJECT)).thenReturn(Optional.of(content));

    Set<String> deletedPaths = underTest.deleteComponent(component);
    assertEquals(singleton(ZIPBALL_PATH), deletedPaths);
  }

  @Test
  public void testDeleteComponentLast() throws IOException {
    when(hostedFacet.rebuildPackageJson(VENDOR, PROJECT)).thenReturn(Optional.empty());
    when(hostedFacet.rebuildProviderJson(VENDOR, PROJECT)).thenReturn(Optional.empty());

    Set<String> deletedPaths = underTest.deleteComponent(component);
    assertEquals(new HashSet<>(Arrays.asList(ZIPBALL_PATH, PROVIDER_PATH, PACKAGE_PATH)), deletedPaths);
  }

  @Test
  public void testDeleteProxyComponent() {
    when(repository.optionalFacet(ComposerHostedFacet.class)).thenReturn(Optional.empty());
    Set<String> deletedPaths = underTest.deleteComponent(component);
    assertEquals(singleton(ZIPBALL_PATH), deletedPaths);
  }
}
