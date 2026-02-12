
package org.globalart.astrum.repository.content.security.internal;

import java.util.Map;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.selector.VariableSource;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SimpleVariableResolverAdapterTest
{
  private static final String FORMAT_VARIABLE = "format";

  private static final String PATH_VARIABLE = "path";

  private static final String TEST_PATH_WITHOUT_SLASH = "some/path.txt";

  private static final String TEST_PATH_WITH_SLASH = '/' + TEST_PATH_WITHOUT_SLASH;

  private static final String TEST_FORMAT = "test";

  @Mock
  Request request;

  @Mock
  Map<String, Object> sourceLookupAsset;

  @Mock
  Repository repository;

  @Mock
  FluentAsset asset;

  @Test
  public void testFromRequest() throws Exception {
    when(request.getPath()).thenReturn(TEST_PATH_WITH_SLASH);
    when(repository.getName()).thenReturn("SimpleVariableResolverAdapterTest");
    when(repository.getFormat()).thenReturn(new Format(TEST_FORMAT)
    {
    });
    SimpleVariableResolverAdapter simpleVariableResolverAdapter = new SimpleVariableResolverAdapter();
    VariableSource source = simpleVariableResolverAdapter.fromRequest(request, repository);

    assertThat(source.getVariableSet(), containsInAnyOrder(FORMAT_VARIABLE, PATH_VARIABLE));
    assertThat(source.get(FORMAT_VARIABLE).get(), is(TEST_FORMAT));
    assertThat(source.get(PATH_VARIABLE).get(), is(TEST_PATH_WITH_SLASH));
  }

  @Test
  public void testFromAsset() throws Exception {
    when(asset.path()).thenReturn(TEST_PATH_WITH_SLASH);
    when(asset.repository()).thenReturn(repository);
    when(repository.getFormat()).thenReturn(new Format(TEST_FORMAT)
    {
    });

    SimpleVariableResolverAdapter simpleVariableResolverAdapter = new SimpleVariableResolverAdapter();
    VariableSource source = simpleVariableResolverAdapter.fromAsset(asset);

    assertThat(source.getVariableSet(), containsInAnyOrder(FORMAT_VARIABLE, PATH_VARIABLE));
    assertThat(source.get(FORMAT_VARIABLE).get(), is(TEST_FORMAT));
    assertThat(source.get(PATH_VARIABLE).get(), is(TEST_PATH_WITH_SLASH));
  }

  @Test
  public void testFromSourceLookup() throws Exception {
    when(sourceLookupAsset.get("name")).thenReturn(TEST_PATH_WITHOUT_SLASH);

    SimpleVariableResolverAdapter simpleVariableResolverAdapter = new SimpleVariableResolverAdapter();
    VariableSource source = simpleVariableResolverAdapter.fromSourceLookup(TEST_FORMAT, sourceLookupAsset);

    assertThat(source.getVariableSet(), containsInAnyOrder(FORMAT_VARIABLE, PATH_VARIABLE));
    assertThat(source.get(FORMAT_VARIABLE).get(), is(TEST_FORMAT));
    assertThat(source.get(PATH_VARIABLE).get(), is(TEST_PATH_WITHOUT_SLASH));
  }
}
