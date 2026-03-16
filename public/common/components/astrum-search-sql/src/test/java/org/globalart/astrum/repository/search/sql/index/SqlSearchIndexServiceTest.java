
package org.globalart.astrum.repository.search.sql.index;

import java.util.List;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.search.sql.store.SearchRecordData;
import org.globalart.astrum.repository.search.sql.store.SearchStore;

import org.junit.jupiter.api.Test;
import org.mockito.Answers;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SqlSearchIndexServiceTest
{
  @Mock
  SearchRecordProducer searchRecordProducer;

  @Mock
  SearchStore searchStore;

  @Mock
  FluentComponents fluentComponents;

  @Mock
  ContentFacet content;

  @Mock
  FluentComponent component;

  @Mock
  SearchRecordData searchTableData;

  @Mock(answer = Answers.RETURNS_MOCKS)
  Format format;

  @Mock
  Repository repository;

  @Mock
  EventManager eventManager;

  @InjectMocks
  SqlSearchIndexService underTest;

  @Test
  void testIndex() {
    when(repository.getFormat()).thenReturn(format);
    when(repository.facet(ContentFacet.class)).thenReturn(content);
    when(content.components()).thenReturn(fluentComponents);
    // One known component, one component is missing and should be deleted
    when(fluentComponents.find(any())).thenReturn(Optional.of(component), Optional.empty());
    when(searchRecordProducer.createSearchRecord(any(), eq(repository))).thenReturn(Optional.of(searchTableData));

    underTest.index(List.of(mockEntityId(0), mockEntityId(1)), repository);

    // known component should be indexed and saved
    verify(searchRecordProducer).createSearchRecord(component, repository);
    verify(searchStore).save(any());
    // unknown component should be deleted
    int internalId = InternalIds.toInternalId(mockEntityId(1));
    verify(searchStore).deleteComponentIds(any(), eq(Set.of(internalId)), any());
    verify(searchStore).deleteSearchAssets(any(), eq(Set.of(internalId)), any());
    verifyNoMoreInteractions(searchRecordProducer, searchStore);
  }

  @Test
  void testIndexBatch_skipsWhenAllComponentsHaveNoAssets() {
    List<FluentComponent> components = List.of(mockComponent(), mockComponent(), mockComponent());
    components.forEach(c -> when(searchRecordProducer.createSearchRecord(c, repository)).thenReturn(Optional.empty()));

    underTest.indexBatch(components, repository);

    verify(searchStore, never()).saveBatch(any());
    verifyNoMoreInteractions(searchStore);
  }

  @Test
  void testIndexBatch_handlesException() {
    List<FluentComponent> components = List.of(mockComponent());
    when(searchRecordProducer.createSearchRecord(components.get(0), repository))
        .thenReturn(Optional.of(searchTableData));

    doThrow(RuntimeException.class).when(searchStore).saveBatch(any());
    doThrow(RuntimeException.class).when(searchStore).save(any());

    underTest.indexBatch(components, repository);

    // this is really a sanity check, the real test is that no exceptions bubble up
    verify(searchStore).save(searchTableData);
  }

  private static EntityId mockEntityId(final int id) {
    EntityId entityId = mock(EntityId.class);
    lenient().when(entityId.getValue()).thenReturn(String.valueOf(id));
    return entityId;
  }

  private static FluentComponent mockComponent() {
    return mock(FluentComponent.class);
  }
}
