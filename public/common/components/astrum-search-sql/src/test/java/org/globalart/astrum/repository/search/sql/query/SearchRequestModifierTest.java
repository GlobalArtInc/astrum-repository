
package org.globalart.astrum.repository.search.sql.query;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContribution;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.Mockito;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsNull.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchRequestModifierTest
{
  @Mock
  private SqlSearchQueryContribution defaultHandler;

  @Mock
  private SqlSearchQueryContribution groupRawHandler;

  @Mock
  private SqlSearchQueryContribution versionHandler;

  private SearchRequestModifier underTest;

  MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = Mockito.mockStatic(QualifierUtil.class);
    Map<String, SqlSearchQueryContribution> handlers = new HashMap<>();

    handlers.put("default", defaultHandler);
    handlers.put("group.raw", groupRawHandler);
    handlers.put("version", versionHandler);
    when(QualifierUtil.buildQualifierBeanMap(Mockito.<List<SqlSearchQueryContribution>>any())).thenReturn(handlers);
    when(defaultHandler.createPredicate(any())).thenReturn(Optional.empty());

    underTest = new SearchRequestModifier(List.of(defaultHandler, groupRawHandler, versionHandler));
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void shouldModifyTheRequest() {
    SearchFilter versionFilter = new SearchFilter("version", "4.13 3.2.0");
    SearchFilter groupRawFilter = new SearchFilter("group.raw", "");

    SearchRequest request = request(versionFilter, groupRawFilter);
    SearchRequest modifiedRequestMock = mock(SearchRequest.class);

    // Modifies the request
    when(versionHandler.modifyRequest(request)).thenReturn(modifiedRequestMock);

    // Returns the same request
    when(groupRawHandler.modifyRequest(request)).thenReturn(request);
    when(groupRawHandler.modifyRequest(modifiedRequestMock)).thenReturn(modifiedRequestMock);

    SearchRequest modifiedRequest = underTest.modify(request);

    assertThat(modifiedRequest, is(notNullValue()));
    assertThat(modifiedRequest, is(modifiedRequestMock));
    verify(versionHandler).modifyRequest(request);
    verify(groupRawHandler).modifyRequest(modifiedRequestMock);
    verifyNoInteractions(defaultHandler);
    verifyNoMoreInteractions(versionHandler, groupRawHandler);
  }

  @Test
  public void shouldModifyTheRequestInDifferentOrder() {
    SearchFilter versionFilter = new SearchFilter("version", "4.13 3.2.0");
    SearchFilter groupRawFilter = new SearchFilter("group.raw", "");

    // Changing the filters order should not affect the result
    SearchRequest request = request(groupRawFilter, versionFilter);
    SearchRequest modifiedRequestMock = mock(SearchRequest.class);

    // Modifies the request
    when(versionHandler.modifyRequest(request)).thenReturn(modifiedRequestMock);

    // Returns the same request
    when(groupRawHandler.modifyRequest(request)).thenReturn(request);
    when(groupRawHandler.modifyRequest(modifiedRequestMock)).thenReturn(modifiedRequestMock);

    SearchRequest modifiedRequest = underTest.modify(request);

    assertThat(modifiedRequest, is(notNullValue()));
    assertThat(modifiedRequest, is(modifiedRequestMock));
    verify(groupRawHandler).modifyRequest(request);
    verify(versionHandler).modifyRequest(request);
    verifyNoInteractions(defaultHandler);
    verifyNoMoreInteractions(versionHandler, groupRawHandler);
  }

  @Test
  public void shouldReturnTheSameRequestWhenNoFiltersArePresent() {
    SearchRequest request = request();

    SearchRequest modifiedRequestMock = mock(SearchRequest.class);
    when(versionHandler.modifyRequest(request)).thenReturn(modifiedRequestMock);

    SearchRequest modifiedRequest = underTest.modify(request);

    assertThat(modifiedRequest, is(notNullValue()));
    assertThat(modifiedRequest, is(request));
    verifyNoInteractions(defaultHandler, versionHandler, groupRawHandler);
  }

  @Test
  public void shouldUseCorrectHandler() {
    // This filter modifies the request
    SearchFilter versionFilter = new SearchFilter("version", "4.13 3.2.0");

    SearchRequest request = request(versionFilter);
    SearchRequest modifiedRequestMock = mock(SearchRequest.class);
    when(versionHandler.modifyRequest(request)).thenReturn(modifiedRequestMock);

    SearchRequest modifiedRequest = underTest.modify(request);

    assertThat(modifiedRequestMock, is(modifiedRequest));
    verify(versionHandler).modifyRequest(request);
    verifyNoInteractions(defaultHandler, groupRawHandler);
  }

  @Test
  public void shouldUseDefaultHandlerWhenPropertyNotFound() {
    SearchFilter versionFilter = new SearchFilter("undefinedProperty", "4.13 3.2.0");

    SearchRequest request = request(versionFilter);
    SearchRequest modifiedRequestMock = mock(SearchRequest.class);
    when(defaultHandler.modifyRequest(request)).thenReturn(modifiedRequestMock);

    SearchRequest modifiedRequest = underTest.modify(request);

    assertThat(modifiedRequestMock, is(modifiedRequest));
    verify(defaultHandler).modifyRequest(request);
    verifyNoInteractions(versionHandler, groupRawHandler);
  }

  private static SearchRequest request(final SearchFilter... filters) {
    return SearchRequest.builder().searchFilters(Arrays.asList(filters)).build();
  }
}
