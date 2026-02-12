
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.Collections;
import java.util.function.Function;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class FluentComponentQueryImplTest
{
  @Mock
  private ContentFacetSupport facet;

  @Mock
  private ComponentStore<?> componentStore;

  @Mock
  private Repository repository;

  @Mock
  private Function<String, String> versionNormalizer;

  @InjectMocks
  private FluentComponentsImpl components;

  @Before
  public void setUp() {
    Format format = new Format("maven2")
    {
    };
    when(repository.getFormat()).thenReturn(format);
    when(repository.getType()).thenReturn(new HostedType());
    when(facet.repository()).thenReturn(repository);
    when(facet.contentRepositoryId()).thenReturn(1);
  }

  @Test
  public void testCountByKind() {
    when(componentStore.countComponents(eq(1), eq("DOCKER"), isNull(), isNull())).thenReturn(5);
    FluentComponentQueryImpl query = new FluentComponentQueryImpl(components, "DOCKER");
    assertThat(query.count(), is(5));
  }

  @Test
  public void testCountByFilter() {
    when(componentStore.countComponents(eq(1), isNull(), eq("name = :name"), any())).thenReturn(3);
    FluentComponentQueryImpl query = new FluentComponentQueryImpl(
        components, "name = :name", Collections.singletonMap("name", "test"));
    assertThat(query.count(), is(3));
  }

  @Test
  public void testBrowseByKind() {
    Continuation<Component> continuation = mock();
    when(componentStore.browseComponents(anyInt(), anyInt(), anyString(), eq("DOCKER"), isNull(), isNull()))
        .thenReturn(continuation);

    FluentComponentQueryImpl query = new FluentComponentQueryImpl(components, "DOCKER");
    Continuation<FluentComponent> result = query.browse(10, "token");
    assertThat(result, is(notNullValue()));
  }

  @Test
  public void testBrowseEager() {
    Continuation<ComponentData> continuation = mock();
    when(componentStore.browseComponentsEager(any(), anyInt(), anyString(), eq("DOCKER"), isNull(), isNull()))
        .thenReturn(continuation);

    FluentComponentQueryImpl query = new FluentComponentQueryImpl(components, "DOCKER");
    Continuation<FluentComponent> result = query.browseEager(10, "token");
    assertThat(result, is(notNullValue()));
  }
}
