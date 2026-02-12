
package org.globalart.astrum.repository.content.rest.internal.resources;

import java.util.List;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.fluent.FluentContinuation;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentImpl;
import org.globalart.astrum.repository.content.maintenance.MaintenanceService;
import org.globalart.astrum.repository.content.rest.ComponentsResourceExtension;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.rest.api.ComponentXO;
import org.globalart.astrum.repository.rest.api.ComponentXOFactory;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.selector.ContentAuthHelper;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.upload.UploadManager;

import com.google.common.collect.ImmutableSet;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.stream.Collectors.toList;
import static java.util.stream.IntStream.range;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.collection.IsCollectionWithSize.hasSize;
import static org.hamcrest.collection.IsEmptyCollection.empty;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.PAGE_SIZE_LIMIT;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComponentsResourceSupportTest
{
  private static final String A_FORMAT = "A_Format";

  // tests assume we've stored more assets than the page limit
  private static final int NUMBER_OF_COMPONENTS = PAGE_SIZE_LIMIT + 2;

  private static final String COMPONENT_NAME = "junit";

  private static final String REPOSITORY_NAME = "repository1";

  private static final String REPOSITORY_URL = "http://localhost:8081/repository/" + REPOSITORY_NAME;

  private static final int COMPONENT_ID = 1001;

  private static final int A_REPOSITORY_ID = 1;

  @Mock
  private ContentFacetSupport contentFacetSupport;

  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private ContentAuthHelper contentAuthHelper;

  @Mock
  private RepositoryManagerRESTAdapter repositoryManagerRESTAdapter;

  @Mock
  private MaintenanceService maintenanceService;

  @Mock
  private UploadManager uploadManager;

  @Mock
  private ComponentXOFactory componentXOFactory;

  @Mock
  private FluentComponents fluentComponents;

  @Mock
  private Format aFormat;

  @Mock
  private Continuation<FluentComponent> componentContinuation;

  @Spy
  private ComponentsResourceExtension componentsResourceExtension = new TestComponentsResourceExtension();

  private ComponentsResource underTest;

  @Before
  public void setup() {
    mockRepository();
    mockContentFacet();
    mockFluentComponents();

    underTest = new ComponentsResource(repositoryManagerRESTAdapter, maintenanceService, uploadManager,
        componentXOFactory, contentAuthHelper, ImmutableSet.of(componentsResourceExtension), null);
  }

  @Test
  public void browseShouldBeEmptyWhenNoComponents() {
    when(componentContinuation.isEmpty()).thenReturn(true);

    List<FluentComponent> components = underTest.browse(repository, null);

    assertThat(components, empty());
    verify(fluentComponents).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, never()).checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void browseShouldBeEmptyWhenNoPermittedComponents() {
    when(componentContinuation.isEmpty()).thenReturn(false).thenReturn(true);
    when(contentAuthHelper.checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME)).thenReturn(false);

    List<FluentComponent> components = underTest.browse(repository, null);

    assertThat(components, empty());
    verify(fluentComponents, times(2)).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, times(NUMBER_OF_COMPONENTS))
        .checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void browseShouldReturnPermittedComponents() {
    int numberOfComponentsNotPermitted = 4;
    int numberOfPermittedComponents = NUMBER_OF_COMPONENTS - numberOfComponentsNotPermitted;

    when(componentContinuation.isEmpty()).thenReturn(false).thenReturn(true);
    when(contentAuthHelper.checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME))
        .thenReturn(false)
        .thenReturn(false, false, false, true);

    List<FluentComponent> components = underTest.browse(repository, null);

    assertThat(components, hasSize(numberOfPermittedComponents));
    verify(fluentComponents, times(2)).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, times(NUMBER_OF_COMPONENTS))
        .checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void shouldTrimNumberOfComponentsToLimit() {
    when(componentContinuation.isEmpty()).thenReturn(false);
    when(contentAuthHelper.checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME)).thenReturn(true);

    List<FluentComponent> components = underTest.browse(repository, null);

    assertThat(components, hasSize(AssetsResource.PAGE_SIZE_LIMIT));
    verify(contentAuthHelper, times(NUMBER_OF_COMPONENTS))
        .checkPathPermissions(COMPONENT_NAME, A_FORMAT, REPOSITORY_NAME);
  }

  private void mockRepository() {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.getName()).thenReturn(REPOSITORY_NAME);
    when(repository.getUrl()).thenReturn(REPOSITORY_URL);
    when(repository.getFormat()).thenReturn(aFormat);
    when(repository.getType()).thenReturn(new HostedType());
    when(aFormat.getValue()).thenReturn(A_FORMAT);
  }

  private void mockContentFacet() {
    when(contentFacet.components()).thenReturn(fluentComponents);
    when(contentFacet.contentRepositoryId()).thenReturn(A_REPOSITORY_ID);
  }

  private void mockFluentComponents() {
    when(fluentComponents.browse(PAGE_SIZE_LIMIT, null))
        .thenReturn(new FluentContinuation<>(componentContinuation, asset -> aFluentComponent()));

    List<FluentComponent> fluentComponentList =
        range(0, NUMBER_OF_COMPONENTS).mapToObj(i -> aFluentComponent()).collect(toList());
    when(componentContinuation.iterator()).thenReturn(fluentComponentList.iterator());
  }

  private FluentComponent aFluentComponent() {
    return new FluentComponentImpl(contentFacetSupport, aComponent());
  }

  private ComponentData aComponent() {
    ComponentData componentData = new ComponentData();
    componentData.setComponentId(COMPONENT_ID);
    componentData.setName(COMPONENT_NAME);
    return componentData;
  }

  private class TestComponentsResourceExtension
      implements ComponentsResourceExtension
  {
    @Override
    public ComponentXO updateComponentXO(final ComponentXO componentXO, final FluentComponent component) {
      return componentXO;
    }
  }
}
