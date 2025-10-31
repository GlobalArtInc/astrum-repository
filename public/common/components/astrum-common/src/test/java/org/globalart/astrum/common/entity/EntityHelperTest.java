
package org.globalart.astrum.common.entity;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

/**
 * Tests for {@link EntityHelper}
 */
public class EntityHelperTest

{
  @Test
  public void testEntityWithoutMetadata() {
    AbstractEntity entity = new AbstractEntity()
    {
    };
    assertFalse(EntityHelper.hasMetadata(entity));

    assertThrows(IllegalStateException.class, () -> EntityHelper.metadata(entity));

    assertThrows(IllegalStateException.class, () -> EntityHelper.isDetached(entity));

    assertThrows(IllegalStateException.class, () -> EntityHelper.id(entity));

    assertThrows(IllegalStateException.class, () -> EntityHelper.version(entity));
  }

  @Test
  public void testEntityWithMetadata() {
    AbstractEntity entity = new AbstractEntity()
    {
    };
    entity.setEntityMetadata(new DetachedEntityMetadata(new DetachedEntityId("a"), new DetachedEntityVersion("1")));

    assertTrue(EntityHelper.hasMetadata(entity));
    assertThat(EntityHelper.metadata(entity), notNullValue());
    assertTrue(EntityHelper.isDetached(entity));
    assertThat(EntityHelper.id(entity).getValue(), is("a"));
    assertThat(EntityHelper.version(entity).getValue(), is("1"));
  }
}
