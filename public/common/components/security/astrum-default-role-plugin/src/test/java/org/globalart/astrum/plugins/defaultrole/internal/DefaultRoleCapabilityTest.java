
package org.globalart.astrum.plugins.defaultrole.internal;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.app.ManagedLifecycleManager;
import org.globalart.astrum.plugins.defaultrole.DefaultRoleRealm;
import org.globalart.astrum.security.authz.AuthorizationConfigurationChanged;
import org.globalart.astrum.security.realm.RealmManager;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.Mockito;
import org.mockito.junit.MockitoJUnitRunner;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class DefaultRoleCapabilityTest
{
  private DefaultRoleCapability underTest;

  @Mock
  private RealmManager realmManager;

  @Mock
  private DefaultRoleRealm defaultRoleRealm;

  @Mock
  private DefaultRoleCapabilityConfiguration configuration;

  @Mock
  private EventManager eventManager;

  private MockedStatic<ManagedLifecycleManager> lifecycleMock;

  @Before
  public void setup() {
    underTest = new DefaultRoleCapability(realmManager, defaultRoleRealm, eventManager);
  }

  @Test
  public void testOnActivate_setsRoleEnablesRealmAndInvalidatesCaches() {
    when(configuration.getRole()).thenReturn("as-admin");

    underTest.onActivate(configuration);

    verify(defaultRoleRealm).setRole("as-admin");
    verify(realmManager).enableRealm(DefaultRoleRealm.NAME);
    verify(eventManager).post(any(AuthorizationConfigurationChanged.class));
  }

  @Before
  public void setupLifecycleMock() {
    lifecycleMock = Mockito.mockStatic(ManagedLifecycleManager.class);
    // Default to "not shutting down" so tests only opt into the shutdown branch when they need to.
    lifecycleMock.when(ManagedLifecycleManager::isShuttingDown).thenReturn(false);
  }

  @After
  public void tearDown() {
    if (lifecycleMock != null) {
      lifecycleMock.close();
    }
  }

  @Test
  public void testRenderDescription_returnsNullWhenNoRoleSet() {
    when(defaultRoleRealm.getRole()).thenReturn(null);

    assertThat(underTest.renderDescription()).isNull();
  }

  @Test
  public void testRenderDescription_returnsRoleNameWhenRoleSet() {
    when(defaultRoleRealm.getRole()).thenReturn("as-admin");

    assertThat(underTest.renderDescription()).isEqualTo("as-admin");
  }

  /**
   * Admin toggles the capability off during normal operation. The realm must be removed from the
   * active realms list and the in-memory role assignment cleared.
   */
  @Test
  public void testOnPassivate_duringNormalOperation_disablesRealmAndClearsRole() {
    underTest.onPassivate(configuration);

    verify(defaultRoleRealm).setRole(null);
    verify(realmManager).disableRealm(DefaultRoleRealm.NAME);
    verify(eventManager).post(any(AuthorizationConfigurationChanged.class));
  }

  /**
   * JVM is shutting down. The capability MUST NOT mutate the persisted realm list — doing so on
   * shutdown corrupts the configuration for the next boot and, in HA, broadcasts the corruption to
   * peer nodes (ASTRUM-53486). This test asserts the shutdown guard is honored.
   */
  @Test
  public void testOnPassivate_duringShutdown_isInert() {
    lifecycleMock.when(ManagedLifecycleManager::isShuttingDown).thenReturn(true);

    underTest.onPassivate(configuration);

    verifyNoInteractions(defaultRoleRealm);
    verifyNoInteractions(realmManager);
    verifyNoInteractions(eventManager);
  }

  /**
   * Regression guard for ASTRUM-53486: even when running on a thread named "JettyShutdownThread"
   * (the actual production shutdown thread) the guard must still fire, because it consults the
   * platform lifecycle flag rather than any thread name. If this test ever passes for the wrong
   * reason (thread-name matching), the previous ASTRUM-43484 bug has been reintroduced.
   */
  @Test
  public void testOnPassivate_ASTRUM_53486_shutdownDetectedRegardlessOfThreadName() {
    String originalName = Thread.currentThread().getName();
    try {
      Thread.currentThread().setName("JettyShutdownThread");
      lifecycleMock.when(ManagedLifecycleManager::isShuttingDown).thenReturn(true);

      underTest.onPassivate(configuration);

      verifyNoInteractions(defaultRoleRealm);
      verifyNoInteractions(realmManager);
      verifyNoInteractions(eventManager);
    }
    finally {
      Thread.currentThread().setName(originalName);
    }
  }
}
