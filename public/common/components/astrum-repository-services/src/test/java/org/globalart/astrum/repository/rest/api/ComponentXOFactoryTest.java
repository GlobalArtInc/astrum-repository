
package org.globalart.astrum.repository.rest.api;

import java.util.Map;

import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.CoreMatchers.instanceOf;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.core.IsNull.notNullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComponentXOFactoryTest
{
  @Mock
  private ComponentXODecorator componentXODecorator;

  private ComponentXOFactory underTest;

  @Before
  public void setup() {
    underTest = new ComponentXOFactory(ImmutableSet.of(componentXODecorator));

    ComponentXO componentXO = new DefaultComponentXO();
    when(componentXODecorator.decorate(any(ComponentXO.class))).thenReturn(new TestComponentXO(componentXO));
  }

  @Test
  public void testComponentXO() {
    ComponentXO componentXO = underTest.createComponentXO();
    assertThat(componentXO, notNullValue());
    assertThat(componentXO, instanceOf(TestComponentXO.class));
    verify(componentXODecorator).decorate(any(ComponentXO.class));
    TestComponentXO testComponentXO = (TestComponentXO) componentXO;
    assertThat(testComponentXO.getWrappedObject(), instanceOf(DefaultComponentXO.class));
    assertThat(testComponentXO.getDecoratedExtraJsonAttributes(), hasEntry("foo", "bar"));
  }

  private class TestComponentXO
      extends DecoratedComponentXO
      implements ComponentXO
  {
    TestComponentXO(final ComponentXO componentXO) {
      super(componentXO);
    }

    @Override
    public Map<String, Object> getDecoratedExtraJsonAttributes() {
      return ImmutableMap.of("foo", "bar");
    }
  }
}
