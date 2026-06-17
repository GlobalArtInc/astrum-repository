
package org.globalart.astrum.coreui;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import jakarta.validation.Validator;

import org.globalart.astrum.bootstrap.validation.ValidationConfiguration;
import org.globalart.astrum.common.Description;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.realm.SecurityRealm;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;
import org.globalart.astrum.testcommon.validation.ValidationExtension.ValidationExecutor;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.realm.Realm;
import org.hibernate.validator.internal.engine.constraintvalidation.ConstraintValidatorFactoryImpl;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;
import org.springframework.context.ApplicationContext;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link RealmSettingsComponent}.
 */
@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class RealmSettingsComponentTest
{
  @ValidationExecutor
  private final Validator validator =
      new ValidationConfiguration().validatorFactory(new ConstraintValidatorFactoryImpl()).getValidator();

  @Mock
  private RealmManager realmManager;

  @Mock
  private ApplicationContext applicationContext;

  private RealmSettingsComponent underTest;

  @BeforeEach
  void setUp() {
    underTest = new RealmSettingsComponent(realmManager);
    underTest.setApplicationContext(applicationContext);
  }

  @Test
  void testRead_returnsConfiguredRealms() {
    List<String> configuredRealms = List.of("AstrumAuthenticatingRealm", "AstrumAuthorizingRealm", "LdapRealm");
    when(realmManager.getConfiguredRealmIds()).thenReturn(configuredRealms);

    RealmSettingsXO result = underTest.read();

    assertThat(result, is(notNullValue()));
    assertThat(result.getRealms(), hasSize(3));
    assertThat(result.getRealms().get(0), is("AstrumAuthenticatingRealm"));
    assertThat(result.getRealms().get(1), is("AstrumAuthorizingRealm"));
    assertThat(result.getRealms().get(2), is("LdapRealm"));
    verify(realmManager).getConfiguredRealmIds();
  }

  @Test
  void testRead_emptyRealms() {
    when(realmManager.getConfiguredRealmIds()).thenReturn(List.of());

    RealmSettingsXO result = underTest.read();

    assertThat(result, is(notNullValue()));
    assertThat(result.getRealms(), hasSize(0));
  }

  @Test
  void testReadRealmTypes_returnsRealmBeans() {
    Map<String, Realm> realmBeans = new LinkedHashMap<>();
    realmBeans.put("AstrumAuthenticatingRealm", new AlphaRealm());
    realmBeans.put("LdapRealm", new BetaRealm());
    when(applicationContext.getBeansOfType(Realm.class)).thenReturn(realmBeans);

    List<ReferenceXO> result = underTest.readRealmTypes();

    assertThat(result, hasSize(2));
    // IDs are the bean names from the application context
    // Sorted alphabetically by name (description)
    assertThat(result.get(0).getId(), is("AstrumAuthenticatingRealm"));
    assertThat(result.get(0).getName(), is("Alpha Realm"));
    assertThat(result.get(1).getId(), is("LdapRealm"));
    assertThat(result.get(1).getName(), is("Beta Realm"));
    verify(applicationContext).getBeansOfType(Realm.class);
  }

  @Test
  void testReadRealmTypes_sortedAlphabeticallyIgnoringCase() {
    Map<String, Realm> realmBeans = new LinkedHashMap<>();
    realmBeans.put("ZetaRealmBean", new ZetaRealm());
    realmBeans.put("AlphaRealmBean", new AlphaRealm());
    realmBeans.put("MidRealmBean", new MidRealm());
    when(applicationContext.getBeansOfType(Realm.class)).thenReturn(realmBeans);

    List<ReferenceXO> result = underTest.readRealmTypes();

    assertThat(result, hasSize(3));
    assertThat(result.get(0).getName(), is("Alpha Realm"));
    assertThat(result.get(1).getName(), is("Mid Realm"));
    assertThat(result.get(2).getName(), is("Zeta Realm"));
  }

  @Test
  void testReadRealmTypes_emptyBeans() {
    when(applicationContext.getBeansOfType(Realm.class)).thenReturn(Map.of());

    List<ReferenceXO> result = underTest.readRealmTypes();

    assertThat(result, hasSize(0));
  }

  @Test
  void testUpdate_setsConfiguredRealmsAndReturnsUpdated() {
    RealmSettingsXO settingsXO = new RealmSettingsXO();
    settingsXO.setRealms(List.of("AstrumAuthenticatingRealm", "LdapRealm"));

    // Mock available realms
    SecurityRealm authRealm = new SecurityRealm("AstrumAuthenticatingRealm", "Astrum Authenticating Realm");
    SecurityRealm ldapRealm = new SecurityRealm("LdapRealm", "LDAP Realm");
    when(realmManager.getAvailableRealms()).thenReturn(List.of(authRealm, ldapRealm));

    List<String> updatedRealms = List.of("AstrumAuthenticatingRealm", "LdapRealm");
    when(realmManager.getConfiguredRealmIds()).thenReturn(updatedRealms);

    RealmSettingsXO result = underTest.update(settingsXO);

    verify(realmManager).setConfiguredRealmIds(List.of("AstrumAuthenticatingRealm", "LdapRealm"));
    assertThat(result, is(notNullValue()));
    assertThat(result.getRealms(), hasSize(2));
    assertThat(result.getRealms().get(0), is("AstrumAuthenticatingRealm"));
    assertThat(result.getRealms().get(1), is("LdapRealm"));
  }

  @Test
  void testUpdate_emptyRealmList() {
    RealmSettingsXO settingsXO = new RealmSettingsXO();
    settingsXO.setRealms(List.of());

    when(realmManager.getAvailableRealms()).thenReturn(List.of());
    when(realmManager.getConfiguredRealmIds()).thenReturn(List.of());

    RealmSettingsXO result = underTest.update(settingsXO);

    verify(realmManager).setConfiguredRealmIds(List.of());
    assertThat(result, is(notNullValue()));
    assertThat(result.getRealms(), hasSize(0));
  }

  @Test
  void testUpdate_rejectsUnknownRealmId() {
    RealmSettingsXO settingsXO = new RealmSettingsXO();
    settingsXO.setRealms(List.of("AstrumAuthenticatingRealm", "java.util.Timer"));

    // Mock available realms - only AstrumAuthenticatingRealm exists
    SecurityRealm authRealm = new SecurityRealm("AstrumAuthenticatingRealm", "Astrum Authenticating Realm");
    when(realmManager.getAvailableRealms()).thenReturn(List.of(authRealm));

    assertThrows(ValidationErrorsException.class, () -> underTest.update(settingsXO));

    // Verify that setConfiguredRealmIds was never called with invalid data
    verify(realmManager, never()).setConfiguredRealmIds(any());
  }

  @Test
  void testUpdate_rejectsAllUnknownRealmIds() {
    RealmSettingsXO settingsXO = new RealmSettingsXO();
    settingsXO.setRealms(List.of("java.util.Timer", "com.example.MaliciousRealm"));

    // No available realms match
    when(realmManager.getAvailableRealms()).thenReturn(List.of());

    assertThrows(ValidationErrorsException.class, () -> underTest.update(settingsXO));

    // Verify that setConfiguredRealmIds was never called
    verify(realmManager, never()).setConfiguredRealmIds(any());
  }

  @Test
  void testSetApplicationContext_setsContext() {
    ApplicationContext newContext = mock(ApplicationContext.class);
    Map<String, Realm> realmBeans = Map.of();
    when(newContext.getBeansOfType(Realm.class)).thenReturn(realmBeans);

    underTest.setApplicationContext(newContext);

    // Verify the new context is used by calling readRealmTypes
    underTest.readRealmTypes();
    verify(newContext).getBeansOfType(Realm.class);
  }

  // Stub Realm implementations with @Description for QualifierUtil.description() to work

  @Description("Alpha Realm")
  static class AlphaRealm
      implements Realm
  {
    @Override
    public String getName() {
      return "alpha";
    }

    @Override
    public boolean supports(AuthenticationToken token) {
      return false;
    }

    @Override
    public AuthenticationInfo getAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
      return null;
    }
  }

  @Description("Beta Realm")
  static class BetaRealm
      implements Realm
  {
    @Override
    public String getName() {
      return "beta";
    }

    @Override
    public boolean supports(AuthenticationToken token) {
      return false;
    }

    @Override
    public AuthenticationInfo getAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
      return null;
    }
  }

  @Description("Mid Realm")
  static class MidRealm
      implements Realm
  {
    @Override
    public String getName() {
      return "mid";
    }

    @Override
    public boolean supports(AuthenticationToken token) {
      return false;
    }

    @Override
    public AuthenticationInfo getAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
      return null;
    }
  }

  @Description("Zeta Realm")
  static class ZetaRealm
      implements Realm
  {
    @Override
    public String getName() {
      return "zeta";
    }

    @Override
    public boolean supports(AuthenticationToken token) {
      return false;
    }

    @Override
    public AuthenticationInfo getAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
      return null;
    }
  }
}
