package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.gson.Gson;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

public class LdapServerConverterTest
{
  @Test
  public void extDirectValuesAsTheClassicUiSendsThem() {
    LdapServerXO xo = new Gson().fromJson("{\"protocol\":\"ldaps\",\"host\":\"ldap.example.com\",\"port\":\"636\","
        + "\"useTrustStore\":\"true\",\"authScheme\":\"DIGEST-MD5\",\"connectionTimeout\":\"15\",\"id\":\"\","
        + "\"ldapGroupsAsRoles\":true,\"groupType\":\"static\",\"template\":\"Active Directory\"}", LdapServerXO.class);

    LdapServer server = LdapServerConverter.fromExtDirect(xo);

    assertThat(server.getProtocol(), is(LdapProtocol.LDAPS));
    assertThat(server.getPort(), is(636));
    assertThat(server.isUseTrustStore(), is(true));
    assertThat(server.getAuthScheme(), is(LdapAuthScheme.DIGEST_MD5));
    assertThat(server.getConnectionTimeoutSeconds(), is(15));
    assertThat(server.getConnectionRetryDelaySeconds(), is(LdapServer.DEFAULT_CONNECTION_RETRY_DELAY));
    assertThat(server.getGroupType(), is(LdapGroupType.STATIC));

    LdapServerXO back = LdapServerConverter.toExtDirect(server, 1);
    assertThat(back.getProtocol(), is("ldaps"));
    assertThat(back.getAuthScheme(), is("DIGEST-MD5"));
    assertThat(back.getGroupType(), is("static"));
    assertThat(back.getAuthPassword(), is((String) null));
    assertThat(back.getUrl(), is("ldaps://ldap.example.com:636/"));
  }

  @Test
  public void restAcceptsAnyCaseAndNeverReturnsThePassword() throws Exception {
    ObjectMapper mapper = new ObjectMapper().disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES);
    LdapServerApiXO xo = mapper.readValue("{\"name\":\"main\",\"protocol\":\"ldap\",\"authScheme\":\"CRAM_MD5\","
        + "\"groupType\":\"dynamic\",\"authPassword\":\"pw\"}", LdapServerApiXO.class);
    assertThat(xo.getAuthScheme(), is(LdapAuthScheme.CRAM_MD5));
    assertThat(xo.getGroupType(), is(LdapGroupType.DYNAMIC));
    assertThat(xo.getAuthPassword(), is("pw"));

    LdapServer server = LdapServerConverter.fromApi(xo);
    assertThat(server.getPort(), is(389));
    assertThat(server.isLdapGroupsAsRoles(), is(true));

    String json = mapper.writeValueAsString(LdapServerConverter.toApi(server, 1));
    assertThat(json, not(containsString("authPassword")));
    assertThat(json, containsString("\"authScheme\":\"CRAM_MD5\""));
    assertThat(json, containsString("\"connectionTimeoutSeconds\":30"));
  }

  @Test
  public void storedServerSurvivesJson() throws Exception {
    ObjectMapper mapper = new ObjectMapper();
    LdapServer server = LdapConfigurationManagerTest.server("main");
    server.setAuthPasswordId("secret-1");
    LdapConfigurationStore.Servers servers = new LdapConfigurationStore.Servers(new ArrayList<>(List.of(server)));

    LdapServer read = mapper.readValue(mapper.writeValueAsString(servers), LdapConfigurationStore.Servers.class)
        .getServers().get(0);

    assertThat(read.getName(), is("main"));
    assertThat(read.getAuthScheme(), is(LdapAuthScheme.SIMPLE));
    assertThat(read.getAuthPasswordId(), is("secret-1"));
    assertThat(mapper.writeValueAsString(server), not(containsString("\"url\"")));
  }

  @Test
  public void base64Credentials() {
    assertThat(LdapServerConverter.decodeBase64("u", "w6Rsw7Zu"), is("älön"));
  }
}
