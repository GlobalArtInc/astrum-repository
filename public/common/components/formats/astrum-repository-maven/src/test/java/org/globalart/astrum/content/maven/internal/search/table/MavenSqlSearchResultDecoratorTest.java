
package org.globalart.astrum.content.maven.internal.search.table;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.sql.SearchResult;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.core.Is.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;

public class MavenSqlSearchResultDecoratorTest

{
  private NestedAttributesMap attributes;

  private NestedAttributesMap childAttributes;

  private MavenSqlSearchResultDecorator underTest;

  @Before
  public void setup() {
    underTest = new MavenSqlSearchResultDecorator();

    attributes = new NestedAttributesMap();
    childAttributes = attributes.child(Maven2Format.NAME);
  }

  @Test
  public void shouldAddBaseVersionAnnotationWhenFormatIsMaven() {
    String baseVersion = "1.1.1";
    ComponentSearchResult component = aComponentSearchResult(Maven2Format.NAME);
    SearchResult searchResult = aSearchResult();
    childAttributes.set(P_BASE_VERSION, baseVersion);

    underTest.updateComponent(component, searchResult);

    assertThat(component.getAnnotation(P_BASE_VERSION), is(baseVersion));
  }

  @Test
  public void shouldNotAddBaseVersionAnnotationWhenFormatIsNotMaven() {
    ComponentSearchResult component = aComponentSearchResult("raw");
    SearchResult searchResult = aSearchResult();

    underTest.updateComponent(component, searchResult);

    assertThat(component.getAnnotation(P_BASE_VERSION), is(nullValue()));
  }

  private SearchResult aSearchResult() {
    SearchResult searchResult = mock(SearchResult.class);
    when(searchResult.attributes()).thenReturn(attributes);
    return searchResult;
  }

  private ComponentSearchResult aComponentSearchResult(final String format) {
    ComponentSearchResult component = new ComponentSearchResult();
    component.setFormat(format);
    return component;
  }
}
