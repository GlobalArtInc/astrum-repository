
package org.globalart.astrum.internal.wonderland;

import java.util.Map.Entry;
import java.util.concurrent.atomic.AtomicBoolean;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link LocalAuthTicketCache}.
 */
public class LocalAuthTicketCacheTest
{
  @Test
  public void addRemoveIsRemoved() {
    LocalAuthTicketCache tokens = new LocalAuthTicketCache();
    tokens.add("user", "foo", "AstrumAuthorizingRealm");
    assertThat(tokens.remove("user", "foo", "AstrumAuthorizingRealm"), is(true));
  }

  @Test
  public void neverAddedRemove() {
    LocalAuthTicketCache tokens = new LocalAuthTicketCache();
    assertThat(tokens.remove("user", "foo", "AstrumAuthorizingRealm"), is(false));
  }

  @Test(expected = IllegalStateException.class)
  public void addDoesNotAllowDuplicates() {
    LocalAuthTicketCache tokens = new LocalAuthTicketCache();
    tokens.add("user", "foo", "AstrumAuthorizingRealm");
    // this should fail
    tokens.add("user", "foo", "AstrumAuthorizingRealm");
  }

  @Test
  public void testExpiredRemoveIsFalse() {
    final AtomicBoolean expired = new AtomicBoolean(false);
    LocalAuthTicketCache tokens = new LocalAuthTicketCache()
    {
      @Override
      protected boolean isTokenExpired(final long now, final Entry<UserAuthToken, Long> entry) {
        return expired.get();
      }
    };

    tokens.add("user", "foo", "AstrumAuthorizingRealm");
    assertThat(tokens.remove("user", "foo", "AstrumAuthorizingRealm"), is(true));

    tokens.add("user", "foo", "AstrumAuthorizingRealm");
    // simulate expire
    expired.set(true);
    assertThat(tokens.remove("user", "foo", "AstrumAuthorizingRealm"), is(false));
  }

  @Test
  public void testRemoveFailsIfDifferentUser() {
    LocalAuthTicketCache cache = new LocalAuthTicketCache();
    cache.add("user", "foo", "AstrumAuthorizingRealm");
    assertThat(cache.remove("bad", "foo", "AstrumAuthorizingRealm"), is(false));
  }
}
