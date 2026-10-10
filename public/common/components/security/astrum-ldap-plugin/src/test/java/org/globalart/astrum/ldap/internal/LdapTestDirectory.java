package org.globalart.astrum.ldap.internal;

import java.io.File;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.Base64;

import javax.net.ssl.SSLContext;

import com.unboundid.ldap.listener.InMemoryDirectoryServer;
import com.unboundid.ldap.listener.InMemoryDirectoryServerConfig;
import com.unboundid.ldap.listener.InMemoryListenerConfig;
import com.unboundid.ldap.sdk.ResultCode;
import com.unboundid.util.ssl.KeyStoreKeyManager;
import com.unboundid.util.ssl.SSLUtil;
import com.unboundid.util.ssl.TrustAllTrustManager;
import com.unboundid.util.ssl.TrustStoreTrustManager;
import com.unboundid.util.ssl.cert.ManageCertificates;

/**
 * An in-memory directory for tests:
 * <ul>
 * <li>{@code alice} (password {@code alice-secret}) in {@code developers} and {@code admins}</li>
 * <li>{@code bob}, whose password attribute holds {@code {SSHA}} of {@code bob-secret}, in {@code developers}</li>
 * <li>{@code carol} (password {@code carol-secret}), no groups and a mail address outside example.com</li>
 * <li>{@code dave} (password {@code dave-secret}), one level down in {@code ou=contractors}</li>
 * </ul>
 * The system account is {@code cn=svc,dc=example,dc=com} with password {@code svc-secret}. Group entries are
 * {@code groupOfUniqueNames}; users also carry {@code memberOf}.
 */
class LdapTestDirectory
    implements AutoCloseable
{
  static final String BASE = "dc=example,dc=com";

  static final String SYSTEM_DN = "cn=svc," + BASE;

  static final String SYSTEM_PASSWORD = "svc-secret";

  private final InMemoryDirectoryServer server;

  private final File keyStore;

  private final char[] keyStorePassword;

  LdapTestDirectory() throws Exception {
    keyStore = File.createTempFile("astrum-ldap-test", ".jks");
    keyStore.delete();
    keyStorePassword = "changeit".toCharArray();
    ResultCode generated = ManageCertificates.main(null, OutputStream.nullOutputStream(), System.err,
        "generate-self-signed-certificate", "--keystore", keyStore.getAbsolutePath(), "--keystore-password",
        new String(keyStorePassword), "--keystore-type", "JKS", "--alias", "server-cert", "--subject-dn",
        "CN=localhost", "--subject-alternative-name-dns", "localhost", "--subject-alternative-name-ip-address",
        "127.0.0.1", "--key-algorithm", "EC", "--key-size-bits", "256", "--signature-algorithm", "SHA256withECDSA");
    if (generated != ResultCode.SUCCESS) {
      throw new IllegalStateException("Could not generate a test certificate: " + generated);
    }
    SSLUtil serverSsl = new SSLUtil(new KeyStoreKeyManager(keyStore, keyStorePassword, "JKS", null),
        new TrustAllTrustManager());

    InMemoryDirectoryServerConfig config = new InMemoryDirectoryServerConfig(BASE);
    config.setSchema(null);
    config.addAdditionalBindCredentials(SYSTEM_DN, SYSTEM_PASSWORD);
    config.setListenerConfigs(
        InMemoryListenerConfig.createLDAPConfig("ldap", 0),
        InMemoryListenerConfig.createLDAPSConfig("ldaps", 0, serverSsl.createSSLServerSocketFactory()));
    server = new InMemoryDirectoryServer(config);
    server.add("dn: " + BASE, "objectClass: top", "objectClass: domain", "dc: example");
    server.add("dn: ou=people," + BASE, "objectClass: organizationalUnit", "ou: people");
    server.add("dn: ou=contractors,ou=people," + BASE, "objectClass: organizationalUnit", "ou: contractors");
    server.add("dn: ou=groups," + BASE, "objectClass: organizationalUnit", "ou: groups");
    server.add(person("alice", "Alice Smith", "alice@example.com", "alice-secret", "ou=people",
        "cn=developers,ou=groups," + BASE, "cn=admins,ou=groups," + BASE));
    server.add(person("bob", "Bob Jones", "bob@example.com", ssha("bob-secret"), "ou=people",
        "cn=developers,ou=groups," + BASE));
    server.add(person("carol", "Carol White", "carol@other.org", "carol-secret", "ou=people"));
    server.add(person("dave", "Dave Brown", "dave@example.com", "dave-secret", "ou=contractors,ou=people"));
    server.add("dn: cn=developers,ou=groups," + BASE, "objectClass: groupOfUniqueNames", "cn: developers",
        "uniqueMember: uid=alice,ou=people," + BASE, "uniqueMember: uid=bob,ou=people," + BASE);
    server.add("dn: cn=admins,ou=groups," + BASE, "objectClass: groupOfUniqueNames", "cn: admins",
        "uniqueMember: uid=alice,ou=people," + BASE);
    server.startListening();
  }

  int ldapPort() {
    return server.getListenPort("ldap");
  }

  int ldapsPort() {
    return server.getListenPort("ldaps");
  }

  /**
   * An SSL context that trusts this directory's certificate, as the Astrum trust store would.
   */
  SSLContext trustingContext() throws Exception {
    return new SSLUtil(new TrustStoreTrustManager(keyStore, keyStorePassword, "JKS", true)).createSSLContext();
  }

  void stop() {
    server.shutDown(true);
  }

  @Override
  public void close() {
    stop();
    keyStore.delete();
  }

  /**
   * A server that finds users under {@code ou=people} and maps groups from {@code memberOf}.
   */
  LdapServer dynamicServer(final String name) {
    LdapServer server = new LdapServer();
    server.setId(name + "-id");
    server.setName(name);
    server.setProtocol(LdapProtocol.LDAP);
    server.setHost("localhost");
    server.setPort(ldapPort());
    server.setSearchBase(BASE);
    server.setAuthScheme(LdapAuthScheme.SIMPLE);
    server.setAuthUsername(SYSTEM_DN);
    server.setConnectionTimeoutSeconds(5);
    server.setConnectionRetryDelaySeconds(300);
    server.setMaxIncidentsCount(3);
    server.setUserBaseDn("ou=people");
    server.setUserSubtree(false);
    server.setUserObjectClass("inetOrgPerson");
    server.setUserIdAttribute("uid");
    server.setUserRealNameAttribute("cn");
    server.setUserEmailAddressAttribute("mail");
    server.setLdapGroupsAsRoles(true);
    server.setGroupType(LdapGroupType.DYNAMIC);
    server.setUserMemberOfAttribute("memberOf");
    return server;
  }

  /**
   * A server that maps groups from {@code groupOfUniqueNames} entries under {@code ou=groups}.
   */
  LdapServer staticServer(final String name) {
    LdapServer server = dynamicServer(name);
    server.setGroupType(LdapGroupType.STATIC);
    server.setGroupBaseDn("ou=groups");
    server.setGroupSubtree(false);
    server.setGroupObjectClass("groupOfUniqueNames");
    server.setGroupIdAttribute("cn");
    server.setGroupMemberAttribute("uniqueMember");
    server.setGroupMemberFormat("${dn}");
    return server;
  }

  static char[] systemPassword() {
    return SYSTEM_PASSWORD.toCharArray();
  }

  static String ssha(final String password) throws Exception {
    byte[] salt = "pepper!!".getBytes(StandardCharsets.UTF_8);
    MessageDigest digest = MessageDigest.getInstance("SHA-1");
    digest.update(password.getBytes(StandardCharsets.UTF_8));
    digest.update(salt);
    byte[] hash = digest.digest();
    byte[] value = new byte[hash.length + salt.length];
    System.arraycopy(hash, 0, value, 0, hash.length);
    System.arraycopy(salt, 0, value, hash.length, salt.length);
    return "{SSHA}" + Base64.getEncoder().encodeToString(value);
  }

  private static String[] person(
      final String uid,
      final String cn,
      final String mail,
      final String password,
      final String container,
      final String... groups)
  {
    String[] lines = new String[8 + groups.length];
    lines[0] = "dn: uid=" + uid + "," + container + "," + BASE;
    lines[1] = "objectClass: inetOrgPerson";
    lines[2] = "uid: " + uid;
    lines[3] = "cn: " + cn;
    lines[4] = "sn: " + cn.substring(cn.indexOf(' ') + 1);
    lines[5] = "mail: " + mail;
    lines[6] = "userPassword: " + password;
    lines[7] = "objectClass: person";
    for (int i = 0; i < groups.length; i++) {
      lines[8 + i] = "memberOf: " + groups[i];
    }
    return lines;
  }
}
