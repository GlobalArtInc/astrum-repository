
package org.globalart.astrum.internal.selector;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.cache.Cache;
import javax.cache.configuration.Factory;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.anonymous.AnonymousPrincipalCollection;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.selector.CselSelector;
import org.globalart.astrum.selector.JexlSelector;
import org.globalart.astrum.selector.Selector;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.selector.SelectorEvaluationException;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.VariableSource;

import com.google.common.collect.ImmutableSet;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.apache.shiro.subject.Subject;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static com.google.common.collect.Sets.newHashSet;
import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.atMostOnce;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor.P_REPOSITORY;
import static org.globalart.astrum.repository.security.RepositorySelector.ALL;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SelectorManagerImplTest
{
  @Mock
  private SelectorConfigurationStore store;

  @Mock
  private SecuritySystem securitySystem;

  @Mock
  private SelectorFactory selectorFactory;

  @Mock
  private AuthorizationManager authorizationManager;

  @Mock
  private User user;

  @Mock
  private VariableSource variableSource;

  @Mock
  private CacheHelper cacheHelper;

  @Mock
  private Time userCacheTimeout;

  @Mock
  private Subject subject;

  @Mock
  private Cache<Object, Object> userCache;

  private SelectorManagerImpl manager;

  private List<SelectorConfiguration> selectorConfigurations;

  private final List<Privilege> privileges = new ArrayList<>();

  @Before
  public void setUp() throws Exception {
    this.manager = new SelectorManagerImpl(store, securitySystem, selectorFactory, cacheHelper, userCacheTimeout);

    when(securitySystem.getAuthorizationManager(DEFAULT_SOURCE)).thenReturn(authorizationManager);
    when(securitySystem.listRoles(UserManager.DEFAULT_SOURCE)).thenReturn(new HashSet<>());
    when(securitySystem.getSubject()).thenReturn(subject);

    when(subject.isAuthenticated()).thenReturn(true);
    when(subject.getPrincipal()).thenReturn("user");
    when(subject.getPrincipals()).thenReturn(new SimplePrincipalCollection("user", "default"));

    when(cacheHelper.maybeCreateCache(anyString(), any(Factory.class))).thenReturn(userCache);

    when(userCache.get(any())).thenReturn(user);

    selectorConfigurations = new ArrayList<>();

    when(store.browse()).thenReturn(selectorConfigurations);

    Selector alwaysTrueSelector = mock(Selector.class);
    when(alwaysTrueSelector.evaluate(variableSource)).thenReturn(true);
    Selector alwaysFalseSelector = mock(Selector.class);
    when(alwaysFalseSelector.evaluate(variableSource)).thenReturn(false);

    when(selectorFactory.createSelector(JexlSelector.TYPE, "true")).thenReturn(alwaysTrueSelector);
    when(selectorFactory.createSelector(JexlSelector.TYPE, "false")).thenReturn(alwaysFalseSelector);
    manager.start();
  }

  @Test
  public void testEvaluate_True() throws Exception {
    SelectorConfiguration selectorConfiguration = getSelectorConfiguration(JexlSelector.TYPE, "true");
    assertThat(manager.evaluate(selectorConfiguration, variableSource), is(true));
  }

  @Test
  public void testEvaluate_False() throws Exception {
    SelectorConfiguration selectorConfiguration = getSelectorConfiguration(JexlSelector.TYPE, "false");
    assertThat(manager.evaluate(selectorConfiguration, variableSource), is(false));
  }

  @Test(expected = SelectorEvaluationException.class)
  public void testEvaluate_InvalidSelectorType() throws Exception {
    SelectorConfiguration selectorConfiguration = getSelectorConfiguration("junk", "");
    manager.evaluate(selectorConfiguration, variableSource);
  }

  @Test
  public void testBrowseJexl() {
    List<SelectorConfiguration> configs = asList(getSelectorConfiguration(JexlSelector.TYPE, "true"),
        getSelectorConfiguration(CselSelector.TYPE, "bar"));
    selectorConfigurations.addAll(configs);
    assertThat(manager.browseJexl(), is(asList(getSelectorConfiguration(JexlSelector.TYPE, "true"))));
  }

  @Test
  public void browseActiveReturnsAllContentSelectorsForMatchingNestedRoles() throws Exception {
    createSelectorConfiguration("roleId", "rolePrivilegeId", "roleSelectorName", "repository");
    securitySystem.listRoles(UserManager.DEFAULT_SOURCE).stream().findFirst().get().getRoles().add("nestedRoleId");
    SelectorConfiguration nestedRoleConfig = createSelectorConfiguration("nestedRoleId", "nestedRolePrivilegeId",
        "nestedRoleSelectorName", ALL);
    when(user.getRoles()).thenReturn(newHashSet(new RoleIdentifier("", "roleId")));
    when(authorizationManager.getPrivileges(ImmutableSet.of("rolePrivilegeId", "nestedRolePrivilegeId")))
        .thenReturn(privileges);

    List<SelectorConfiguration> selectors = manager.browseActive(asList("anyRepository"), asList("anyFormat"));

    assertThat(selectors, contains(is(nestedRoleConfig)));
  }

  @Test
  public void browseActiveCachesRolesList() throws Exception {
    // initial setup calls securitySystem.listRoles twice times
    createSelectorConfiguration("roleId", "rolePrivilegeId", "roleSelectorName", "repository");
    securitySystem.listRoles(UserManager.DEFAULT_SOURCE).stream().findFirst().get().getRoles().add("nestedRoleId");
    verify(securitySystem, times(2)).listRoles(UserManager.DEFAULT_SOURCE);

    manager.browseActive(asList("anyRepository"), asList("anyFormat"));
    verify(securitySystem, times(3)).listRoles(UserManager.DEFAULT_SOURCE);

    manager.browseActive(asList("anyRepository"), asList("anyFormat"));
    verify(securitySystem, times(3)).listRoles(UserManager.DEFAULT_SOURCE);

    // configuration change clears cache
    createSelectorConfiguration("nestedRoleId", "nestedRolePrivilegeId",
        "nestedRoleSelectorName", ALL);
    manager.browseActive(asList("anyRepository"), asList("anyFormat"));
    verify(securitySystem, times(4)).listRoles(UserManager.DEFAULT_SOURCE);

    manager.browseActive(asList("anyRepository"), asList("anyFormat"));
    verify(securitySystem, times(4)).listRoles(UserManager.DEFAULT_SOURCE);
  }

  @Test
  public void browseActiveReturnsNoContentSelectorsForNonMatchingFormats() throws Exception {
    String repositoryFormat = "format";
    createSelectorConfiguration("roleId", "rolePrivilegeId", "roleSelectorName", ALL + '-' + repositoryFormat);
    createSelectorConfiguration("nestedRoleId", "nestedRolePrivilegeId", "nestedRoleSelectorName",
        ALL + '-' + repositoryFormat);
    when(user.getRoles()).thenReturn(newHashSet(new RoleIdentifier("", "roleId")));
    when(authorizationManager.getPrivileges(ImmutableSet.of("rolePrivilegeId", "nestedRolePrivilegeId")))
        .thenReturn(privileges);

    List<SelectorConfiguration> selectors = manager.browseActive(asList("repository"), asList("unknownFormat"));

    assertThat(selectors.size(), is(0));
  }

  @Test
  public void browseActiveReturnsRepositorySpecificContentSelectorsForMatchingNestedRoles() throws Exception {
    String repositoryName = "repository";
    String repositoryFormat = "format";
    SelectorConfiguration roleConfig = createSelectorConfiguration("roleId", "rolePrivilegeId",
        "roleSelectorName", repositoryName);
    securitySystem.listRoles(UserManager.DEFAULT_SOURCE).stream().findFirst().get().getRoles().add("nestedRoleId");
    SelectorConfiguration nestedRoleConfig = createSelectorConfiguration("nestedRoleId",
        "nestedRolePrivilegeId", "nestedRoleSelectorName", repositoryName);
    when(user.getRoles()).thenReturn(newHashSet(new RoleIdentifier("", "roleId")));
    when(authorizationManager.getPrivileges(ImmutableSet.of("rolePrivilegeId", "nestedRolePrivilegeId")))
        .thenReturn(privileges);

    List<SelectorConfiguration> selectors = manager.browseActive(asList(repositoryName), asList(repositoryFormat));

    assertThat(selectors, containsInAnyOrder(is(roleConfig), is(nestedRoleConfig)));
  }

  @Test
  public void browseActiveReturnsNoContentSelectorsWhenAnonymousAccessDisabled() throws Exception {
    when(securitySystem.currentUser()).thenReturn(null);

    List<SelectorConfiguration> selectors = manager.browseActive(asList("anyRepository"), asList("anyFormat"));

    assertThat(selectors.size(), is(0));
  }

  @Test
  public void testDelete_Succeeds() {
    SelectorConfiguration selectorConfiguration = new SelectorConfigurationData();
    selectorConfiguration.setName("selector");
    selectorConfiguration.setType(CselSelector.TYPE);

    manager.create(selectorConfiguration);
    verify(store).create(selectorConfiguration);

    manager.delete(selectorConfiguration);
    verify(store).delete(selectorConfiguration);
  }

  @Test
  public void testDelete_FailsWhenContentSelectorIsUsedByPrivilege() throws Exception {
    SelectorConfiguration selector = createSelectorConfiguration("role", "privilege", "selector", "repository");

    assertThrows("Content selector selector is in use and cannot be deleted", IllegalStateException.class,
        () -> manager.delete(selector));

    verifyNoInteractions(store);
  }

  @Test
  public void findByNameReturnsEmptyOptional() throws Exception {
    Optional<SelectorConfiguration> configuration = manager.findByName("invalidSelectorName");
    assertThat(configuration.isPresent(), is(false));
  }

  @Test
  public void findByNameReturnsExpectedSelector() throws Exception {
    SelectorConfiguration expectedSelector = getSelectorConfiguration(CselSelector.TYPE, "bar");
    List<SelectorConfiguration> configs = asList(expectedSelector);
    selectorConfigurations.addAll(configs);

    Optional<SelectorConfiguration> configuration = manager.findByName(expectedSelector.getName());

    assertThat(configuration.get(), is(expectedSelector));
  }

  @Test
  public void testUserIsTakenFromCache() throws UserNotFoundException {
    manager.browseActive(null, null);

    verify(userCache, atLeastOnce()).get(any());
    verify(securitySystem, never()).currentUser();
  }

  @Test
  public void testUserIsTakenFromSystemIfNoValueInCache() throws UserNotFoundException {
    when(userCache.get(any())).thenReturn(null);
    when(securitySystem.currentUser()).thenReturn(user);

    manager.browseActive(null, null);

    verify(securitySystem, atMostOnce()).currentUser();
    verify(userCache, atLeastOnce()).put(any(), any());
  }

  @Test
  public void testAnonymousUserIsTakenFromSystemIfNoValueInCache() throws UserNotFoundException {
    when(subject.isAuthenticated()).thenReturn(false);
    when(subject.getPrincipals()).thenReturn(new AnonymousPrincipalCollection("anonymous", "default"));
    when(userCache.get(any())).thenReturn(null);
    when(securitySystem.currentUser()).thenReturn(user);

    manager.browseActive(null, null);

    verify(securitySystem, atMostOnce()).currentUser();
    verify(userCache, atLeastOnce()).put(any(), any());
  }

  private static SelectorConfiguration getSelectorConfiguration(final String type, final String expression) {
    SelectorConfiguration selectorConfiguration = new SelectorConfigurationData();
    Map<String, Object> attributes = new HashMap<>();
    attributes.put("expression", expression);
    selectorConfiguration.setAttributes(attributes);
    selectorConfiguration.setType(type);
    return selectorConfiguration;
  }

  private SelectorConfiguration createSelectorConfiguration(
      final String roleId,
      final String rolePrivilegeId,
      final String roleSelectorName,
      final String repositoryName) throws Exception
  {
    createRole(roleId, rolePrivilegeId);
    createRepositoryContentSelectorPrivilege(rolePrivilegeId, roleSelectorName, repositoryName);

    SelectorConfiguration selectorConfiguration = new SelectorConfigurationData();
    selectorConfiguration.setName(roleSelectorName);

    selectorConfigurations.add(selectorConfiguration);

    return selectorConfiguration;
  }

  private void createRepositoryContentSelectorPrivilege(
      final String privilegeId,
      final String selectorConfigurationName,
      final String repositoryName)
  {
    Privilege privilege = new Privilege();
    privilege.setId(privilegeId);
    privilege.setType(RepositoryContentSelectorPrivilegeDescriptor.TYPE);
    privilege.getProperties()
        .put(RepositoryContentSelectorPrivilegeDescriptor.P_CONTENT_SELECTOR, selectorConfigurationName);
    privilege.getProperties().put(P_REPOSITORY, repositoryName);
    privileges.add(privilege);

    when(securitySystem.listPrivileges()).thenReturn(Collections.singleton(privilege));
  }

  private Role createRole(final String roleId, final String privilegeId) throws Exception {
    Role role = new Role();
    role.setRoleId(roleId);
    role.getPrivileges().add(privilegeId);

    securitySystem.listRoles(UserManager.DEFAULT_SOURCE).add(role);
    when(authorizationManager.getRole(roleId)).thenReturn(role);

    return role;
  }
}
