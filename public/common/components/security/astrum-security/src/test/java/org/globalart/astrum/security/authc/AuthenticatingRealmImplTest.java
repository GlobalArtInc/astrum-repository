
package org.globalart.astrum.security.authc;

import java.nio.charset.StandardCharsets;
import java.security.SecureRandom;
import java.util.HashSet;
import java.util.Set;

import javax.crypto.Cipher;
import javax.crypto.SecretKeyFactory;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.JwtFilter;
import org.globalart.astrum.security.JwtHelper;
import org.globalart.astrum.security.authc.AuthenticatingRealmImplTest.AuthenticatingRealmImplTestConfiguration;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.internal.AuthenticatingRealmImpl;
import org.globalart.astrum.security.internal.SecurityConfigurationManagerImpl;

import com.google.common.hash.Hashing;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.CredentialsException;
import org.apache.shiro.authc.DisabledAccountException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.realm.Realm;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.mockito.ArgumentMatchers.any;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.atMostOnce;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@Import(AuthenticatingRealmImplTestConfiguration.class)
public class AuthenticatingRealmImplTest
    extends AbstractSecurityTest
{
  public static final String ENCRYPTED_PASSWORD_PHC_STRING = "$pbkdf2-sha256$restOfPhcString";

  protected static class AuthenticatingRealmImplTestConfiguration
      extends BaseSecurityConfiguration
  {
    @Primary
    @Bean
    public CryptoHelper cryptoHelper() throws Exception {
      CryptoHelper cryptoHelper = mock(CryptoHelper.class);
      when(cryptoHelper.createSecureRandom()).thenReturn(new SecureRandom());
      SecretKeyFactory keyFactory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA1");
      when(cryptoHelper.createSecretKeyFactory(anyString())).thenReturn(keyFactory);

      when(cryptoHelper.createCipher(anyString())).thenReturn(Cipher.getInstance("AES/CBC/PKCS5Padding"));
      return cryptoHelper;
    }

    @Bean
    public JwtHelper jwtHelper() {
      return mock(JwtHelper.class);
    }

    @Bean
    public JwtFilter jwtFilter() {
      return mock(JwtFilter.class);
    }
  }

  private AuthenticatingRealmImpl realm;

  private SecurityConfigurationManagerImpl configurationManager;

  private CPrivilege testPrivilege;

  private CRole testRole;

  private CUser testUser;

  @BeforeEach
  @Override
  protected void setUp() throws Exception {
    super.setUp();

    realm = (AuthenticatingRealmImpl) lookup(Realm.class, AuthenticatingRealmImpl.NAME);
    configurationManager = lookup(SecurityConfigurationManagerImpl.class);
  }

  @AfterEach
  public void cleanup() throws Exception {
    if (testUser != null) {
      configurationManager.deleteUser(testUser.getId());
    }

    if (testRole != null) {
      configurationManager.deleteRole(testRole.getId());
    }

    if (testPrivilege != null) {
      configurationManager.deletePrivilege(testPrivilege.getId());
    }
  }

  @Test
  public void testSuccessfulAuthentication() throws Exception {
    when(passwordService.encryptPassword(any()))
        .thenReturn(ENCRYPTED_PASSWORD_PHC_STRING);

    when(passwordService.passwordsMatch(any(), anyString())).thenReturn(true);
    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE);

    UsernamePasswordToken upToken = new UsernamePasswordToken("username", "somePassword");
    AuthenticationInfo ai = realm.getAuthenticationInfo(upToken);
    String password = new String((char[]) ai.getCredentials());

    assertThat(password, equalTo(ENCRYPTED_PASSWORD_PHC_STRING));
  }

  @Test
  public void testCreateWithPassword() throws Exception {

    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE);

    String clearPassword = "default-password";
    String username = "testCreateWithPassowrdEmailUserId";

    CUser user = user("testCreateWithPassowrdEmail@somewhere", "testCreateWithPassowrdEmail",
        "testCreateWithPassowrdEmail", CUser.STATUS_ACTIVE, username, null);

    Set<String> roles = new HashSet<String>();
    roles.add("role");

    when(passwordService.passwordsMatch(any(), anyString())).thenReturn(true);

    when(passwordService.encryptPassword(any()))
        .thenReturn(ENCRYPTED_PASSWORD_PHC_STRING);

    configurationManager.createUser(user, clearPassword, roles);

    UsernamePasswordToken upToken = new UsernamePasswordToken("testCreateWithPassowrdEmailUserId", clearPassword);
    AuthenticationInfo ai = realm.getAuthenticationInfo(upToken);
    String password = new String((char[]) ai.getCredentials());

    assertThat(password, equalTo(ENCRYPTED_PASSWORD_PHC_STRING));
  }

  @Test
  public void testFailedAuthentication() throws Exception {
    when(passwordService.encryptPassword(any()))
        .thenAnswer(inv -> "pbkdf2-sha1" + inv.getArgument(0).toString());

    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE);

    UsernamePasswordToken upToken = new UsernamePasswordToken("username", "badpassword");

    assertThrows(IncorrectCredentialsException.class, () -> realm.getAuthenticationInfo(upToken));
  }

  @Test
  public void testDisabledAuthentication() throws Exception {
    when(passwordService.encryptPassword(any()))
        .thenAnswer(inv -> "pbkdf2-sha1" + inv.getArgument(0).toString());

    buildTestAuthenticationConfig(CUser.STATUS_DISABLED);
    UsernamePasswordToken upToken = new UsernamePasswordToken("username", "password");

    assertThrows(DisabledAccountException.class, () -> realm.getAuthenticationInfo(upToken));
  }

  @Test
  public void testGetAuthenticationInfo_userStatusChangePassword() throws Exception {
    when(passwordService.encryptPassword(any()))
        .thenReturn(ENCRYPTED_PASSWORD_PHC_STRING);

    when(passwordService.passwordsMatch(any(), anyString())).thenReturn(true);
    buildTestAuthenticationConfig(CUser.STATUS_CHANGE_PASSWORD);

    UsernamePasswordToken upToken = new UsernamePasswordToken("username", "password");
    AuthenticationInfo ai = realm.getAuthenticationInfo(upToken);
    String password = new String((char[]) ai.getCredentials());
    assertThat(password, equalTo(ENCRYPTED_PASSWORD_PHC_STRING));
    verify(passwordService, atMostOnce()).passwordsMatch("password", password);
  }

  @Test
  public void testDetectLegacyUser() throws Exception {
    String password = "password";
    String username = "username";
    buildLegacyTestAuthenticationConfig(password);

    when(passwordService.passwordsMatch(any(), anyString())).thenReturn(true);
    when(passwordService.encryptPassword(any()))
        .thenReturn(ENCRYPTED_PASSWORD_PHC_STRING);

    UsernamePasswordToken upToken = new UsernamePasswordToken(username, password);
    AuthenticationInfo ai = realm.getAuthenticationInfo(upToken);
    CUser updatedUser = this.configurationManager.readUser(username);
    String hash = new String((char[]) ai.getCredentials());

    assertThat(hash, equalTo(ENCRYPTED_PASSWORD_PHC_STRING));
    assertThat(updatedUser.getPassword(), equalTo(ENCRYPTED_PASSWORD_PHC_STRING));
  }

  @Test
  public void testNoneExistentUser() throws Exception {
    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE);
    UsernamePasswordToken upToken = new UsernamePasswordToken("non-existent-user", "password");

    assertThrows(UnknownAccountException.class, () -> realm.getAuthenticationInfo(upToken));
  }

  @Test
  public void testEmptyPassword() throws Exception {
    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE);
    UsernamePasswordToken upToken = new UsernamePasswordToken("username", (String) null);

    assertThrows(CredentialsException.class, () -> realm.getAuthenticationInfo(upToken));
  }

  private void buildTestAuthenticationConfig(final String status) throws Exception {
    buildTestAuthenticationConfig(status, this.hashPassword("somePassword"));
  }

  private void buildTestAuthenticationConfig(final String status, final String hash) throws Exception {
    CPrivilege priv = new MemoryCPrivilege();
    priv.setId("priv");
    priv.setName("name");
    priv.setDescription("desc");
    priv.setType("method");
    priv.setProperty("method", "read");
    priv.setProperty("permission", "somevalue");

    testPrivilege = priv;
    configurationManager.createPrivilege(priv);

    CRole role = configurationManager.newRole();
    role.setName("name");
    role.setId("role");
    role.setDescription("desc");
    role.addPrivilege("priv");

    testRole = role;
    configurationManager.createRole(role);

    testUser = user("dummyemail@somewhere", "dummyFirstName", "dummyLastName", status, "username", hash);

    Set<String> roles = new HashSet<String>();
    roles.add("role");

    configurationManager.createUser(testUser, roles);
  }

  private String hashPassword(final String password) {
    return passwordService.encryptPassword(password);
  }

  @SuppressWarnings("deprecation")
  private String legacyHashPassword(final String password) {
    return Hashing.sha1().hashString(password, StandardCharsets.UTF_8).toString();
  }

  private void buildLegacyTestAuthenticationConfig(final String password) throws Exception {
    buildTestAuthenticationConfig(CUser.STATUS_ACTIVE, legacyHashPassword(password));
  }

  private static CUser user(
      final String email,
      final String firstName,
      final String lastName,
      final String status,
      final String id,
      final String passwordHash)
  {
    CUser testUser = new MemoryCUser();
    testUser.setEmail(email);
    testUser.setFirstName(firstName);
    testUser.setLastName(lastName);
    testUser.setStatus(status);
    testUser.setId(id);
    testUser.setPassword(passwordHash);
    return testUser;
  }
}
