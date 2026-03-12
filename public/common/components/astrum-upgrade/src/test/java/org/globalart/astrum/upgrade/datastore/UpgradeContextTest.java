
package org.globalart.astrum.upgrade.datastore;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class UpgradeContextTest
{
  private final UpgradeContext underTest = new UpgradeContext();

  @Test
  void unsetFlagIsFalse() {
    assertThat(underTest.isFlagSet("some.flag")).isFalse();
  }

  @Test
  void flag_roundTrips() {
    underTest.setFlag("feature.a");
    assertThat(underTest.isFlagSet("feature.a")).isTrue();
    // independent of other keys
    assertThat(underTest.isFlagSet("feature.b")).isFalse();
  }

  @Test
  void missingValueIsEmpty() {
    assertThat(underTest.get("nope")).isEmpty();
  }

  @Test
  void value_roundTrips() {
    underTest.put("count", 42);
    assertThat(underTest.get("count")).contains(42);
    assertThat(underTest.get("count", Integer.class)).contains(42);
  }

  @Test
  void getTyped_returnsEmptyWhenStoredValueIsNotAssignable() {
    underTest.put("count", 42);
    // a value present but not assignable to the requested type is treated as absent (no ClassCastException)
    assertThat(underTest.get("count", String.class)).isEmpty();
    // the untyped getter still sees the underlying value
    assertThat(underTest.get("count")).contains(42);
  }

  @Test
  void flagIsDistinctFromValueLookup() {
    underTest.setFlag("flag");
    // a raised flag is stored as Boolean.TRUE and is also visible via the generic getter
    assertThat(underTest.get("flag")).contains(Boolean.TRUE);
  }
}
