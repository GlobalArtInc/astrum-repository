
package org.globalart.astrum.selector;

import org.junit.Test;

import static com.google.common.collect.ImmutableMap.of;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class VariableSourceTest
{
  static final String FOO_BAR = "foo.bar";

  static final String FOO = "foo";

  static final String MOO = "moo";

  static final String BAZ = "baz";

  static final String GOO = "goo";

  static final String BAR = "bar";

  @Test
  public void test3ResolversFirstWins() {
    {
      VariableSource source = new VariableSourceBuilder()
          .addResolver(new ConstantVariableResolver(MOO, FOO_BAR))
          .addResolver(new ConstantVariableResolver(BAZ, FOO_BAR))
          .addResolver(new PropertiesResolver<>(FOO, of(BAR, GOO)))
          .build();
      assertEquals(1, source.getVariableSet().size());
      assertTrue(source.getVariableSet().contains(FOO_BAR));
      assertEquals(MOO, source.get(FOO_BAR).get());
    }
    {
      VariableSource source = new VariableSourceBuilder()
          .addResolver(new ConstantVariableResolver(BAZ, FOO_BAR))
          .addResolver(new PropertiesResolver<>(FOO, of(BAR, GOO)))
          .addResolver(new ConstantVariableResolver(MOO, FOO_BAR))
          .build();
      assertEquals(1, source.getVariableSet().size());
      assertTrue(source.getVariableSet().contains(FOO_BAR));
      assertEquals(BAZ, source.get(FOO_BAR).get());
    }
    {
      VariableSource source = new VariableSourceBuilder()
          .addResolver(new PropertiesResolver<>(FOO, of(BAR, GOO)))
          .addResolver(new ConstantVariableResolver(MOO, FOO_BAR))
          .addResolver(new ConstantVariableResolver(BAZ, FOO_BAR))
          .build();
      assertEquals(1, source.getVariableSet().size());
      assertTrue(source.getVariableSet().contains(FOO_BAR));
      assertEquals(GOO, source.get(FOO_BAR).get());
    }
  }

  @Test
  public void testNoResolvers() {
    VariableSource source = new VariableSourceBuilder().build();
    assertEquals(0, source.getVariableSet().size());
    assertFalse(source.getVariableSet().contains(FOO_BAR));
    assertFalse(source.get(FOO_BAR).isPresent());
  }
}
