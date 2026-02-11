
package org.globalart.astrum.repository.content;

import java.util.HashMap;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryAttributeServiceImplTest
{
  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  private RepositoryAttributeServiceImpl underTest;

  @Before
  public void setUp() {
    underTest = new RepositoryAttributeServiceImpl();
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
  }

  @Test
  public void testSetRepositoryAttributeSuccess() {
    when(contentFacet.withAttribute("key1", "value1")).thenReturn(contentFacet);

    boolean result = underTest.setRepositoryAttribute(repository, "key1", "value1");

    assertThat(result, is(true));
    verify(contentFacet).withAttribute("key1", "value1");
  }

  @Test
  public void testSetRepositoryAttributeReturnsFalseOnException() {
    when(contentFacet.withAttribute(any(), any())).thenThrow(new RuntimeException("test"));

    boolean result = underTest.setRepositoryAttribute(repository, "key1", "value1");

    assertThat(result, is(false));
  }

  @Test
  public void testGetRepositoryAttribute() {
    NestedAttributesMap attributes = new NestedAttributesMap("attributes", new HashMap<>());
    attributes.set("myKey", "myValue");
    when(contentFacet.attributes()).thenReturn(attributes);

    String result = underTest.getRepositoryAttribute(repository, "myKey", "default");

    assertThat(result, is("myValue"));
  }

  @Test
  public void testGetRepositoryAttributeReturnsDefaultOnException() {
    when(contentFacet.attributes()).thenThrow(new RuntimeException("test"));

    String result = underTest.getRepositoryAttribute(repository, "missingKey", "fallback");

    assertThat(result, is("fallback"));
  }

  @Test
  public void testGetRepositoryAttributeReturnsNullDefault() {
    when(contentFacet.attributes()).thenThrow(new RuntimeException("test"));

    String result = underTest.getRepositoryAttribute(repository, "missingKey", null);

    assertThat(result, is(nullValue()));
  }
}
