
package org.globalart.astrum.upgrade.datastore;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class LocalUpgradeCooperationTest
{
  @Mock
  private Cooperation2Factory cooperationFactory;

  @Mock
  private Cooperation2Factory.Builder builder;

  @Mock
  private Cooperation2 cooperation;

  @Test
  void get_buildsCooperationFromInjectedFactory() {
    when(cooperationFactory.configure()).thenReturn(builder);
    when(builder.build("my-scope")).thenReturn(cooperation);

    Cooperation2 result = new LocalUpgradeCooperation(cooperationFactory).get("my-scope");

    assertThat(result).isSameAs(cooperation);
  }

  @Test
  void get_memoizesPerId() {
    when(cooperationFactory.configure()).thenReturn(builder);
    when(builder.build("my-scope")).thenReturn(cooperation);
    LocalUpgradeCooperation underTest = new LocalUpgradeCooperation(cooperationFactory);

    Cooperation2 first = underTest.get("my-scope");
    Cooperation2 second = underTest.get("my-scope");

    // repeated get(sameId) returns the memoized instance, and the factory is consulted only once
    assertThat(second).isSameAs(first);
    verify(cooperationFactory, times(1)).configure();
    verify(builder, times(1)).build("my-scope");
  }
}
