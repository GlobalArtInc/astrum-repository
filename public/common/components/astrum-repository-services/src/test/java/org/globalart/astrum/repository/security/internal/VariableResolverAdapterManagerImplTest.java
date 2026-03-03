
package org.globalart.astrum.repository.security.internal;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.security.VariableResolverAdapter;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class VariableResolverAdapterManagerImplTest
{
  @Mock
  private VariableResolverAdapter specializedAdapter;

  @Mock
  private VariableResolverAdapter defaultAdapter;

  private VariableResolverAdapterManagerImpl manager;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setUp() {
    mockedStatic = mockStatic(QualifierUtil.class);
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(
        Map.of(VariableResolverAdapterManagerImpl.DEFAULT_ADAPTER_NAME, defaultAdapter, "special", specializedAdapter));
    manager = new VariableResolverAdapterManagerImpl(List.of(defaultAdapter, specializedAdapter));
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void testGet_SpecializedAdapter() {
    assertThat(manager.get("special"), is(specializedAdapter));
  }

  @Test
  public void testGet_DefaultAdapter() {
    assertThat(manager.get("other-format"), is(defaultAdapter));
  }
}
