
package org.globalart.astrum.internal.email;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.mockito.Mockito;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsFactory;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.mybatis.handlers.SecretTypeHandler;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.testdb.DataSessionRule;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class EmailConfigurationDAOTest
{

  private final SecretsFactory secretsFactory = mock(SecretsFactory.class);

  @Rule
  public DataSessionRule sessionRule =
      new DataSessionRule().access(EmailConfigurationDAO.class).handle(new SecretTypeHandler(secretsFactory));

  private DataSession<?> session;

  private EmailConfigurationDAO dao;

  private Secret secret;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(EmailConfigurationDAO.class);
    secret = mock(Secret.class);
  }

  @After
  public void tearDown() {
    session.close();
  }

  private EmailConfigurationData createEmailConfigurationData(Secret secret) {
    Mockito.when(secret.getId()).thenReturn("_1");
    Mockito.when(secretsFactory.from("_1")).thenReturn(secret);
    EmailConfigurationData entity = new EmailConfigurationData();
    entity.setEnabled(true);
    entity.setHost("localhost");
    entity.setPort(25);
    entity.setUsername("email_user");
    entity.setPassword(secret);
    entity.setFromAddress("foo@example.com");
    entity.setSubjectPrefix("PREFIX: ");
    entity.setStartTlsEnabled(true);
    entity.setStartTlsRequired(false);
    entity.setSslOnConnectEnabled(true);
    entity.setSslCheckServerIdentityEnabled(false);
    entity.setAstrumTrustStoreEnabled(true);
    return entity;
  }

  @Test
  public void testReadWriteSingleEmailConfiguration() {
    EmailConfigurationData entity = createEmailConfigurationData(secret);

    dao.set(entity);
    EmailConfiguration readEntity = dao.get().orElse(null);

    assertThat(readEntity, is(notNullValue()));
    assertThat(readEntity.isEnabled(), is(true));
    assertThat(readEntity.getHost(), is("localhost"));
    assertThat(readEntity.getPort(), is(25));
    assertThat(readEntity.getUsername(), is("email_user"));
    assertThat(readEntity.getPassword(), is(secret));
    assertThat(readEntity.getFromAddress(), is("foo@example.com"));
    assertThat(readEntity.getSubjectPrefix(), is("PREFIX: "));
    assertThat(readEntity.isStartTlsEnabled(), is(true));
    assertThat(readEntity.isStartTlsRequired(), is(false));
    assertThat(readEntity.isSslOnConnectEnabled(), is(true));
    assertThat(readEntity.isSslCheckServerIdentityEnabled(), is(false));
    assertThat(readEntity.isAstrumTrustStoreEnabled(), is(true));
  }

  @Test
  public void testUpdateEmailConfiguration() {
    EmailConfigurationData entity = createEmailConfigurationData(secret);
    Secret otherSecret = mock(Secret.class);
    Mockito.when(otherSecret.getId()).thenReturn("_2");
    Mockito.when(secretsFactory.from("_2")).thenReturn(otherSecret);

    dao.set(entity);
    EmailConfigurationData readEntity = dao.get().orElse(null);

    assertThat(readEntity, is(notNullValue()));
    readEntity.setEnabled(false);
    readEntity.setHost("remotehost");
    readEntity.setPort(26);
    readEntity.setUsername("email_user2");
    readEntity.setPassword(otherSecret);
    readEntity.setFromAddress("bar@example.com");
    readEntity.setSubjectPrefix("XYZ: ");
    readEntity.setStartTlsEnabled(false);
    readEntity.setStartTlsRequired(true);
    readEntity.setSslOnConnectEnabled(false);
    readEntity.setSslCheckServerIdentityEnabled(true);
    readEntity.setAstrumTrustStoreEnabled(false);
    dao.set(readEntity);

    EmailConfigurationData updatedEntity = dao.get().orElse(null);

    assertThat(updatedEntity, is(notNullValue()));
    assertThat(updatedEntity.isEnabled(), is(false));
    assertThat(updatedEntity.getHost(), is("remotehost"));
    assertThat(updatedEntity.getPort(), is(26));
    assertThat(updatedEntity.getUsername(), is("email_user2"));
    assertThat(updatedEntity.getPassword(), is(otherSecret));
    assertThat(updatedEntity.getFromAddress(), is("bar@example.com"));
    assertThat(updatedEntity.getSubjectPrefix(), is("XYZ: "));
    assertThat(updatedEntity.isStartTlsEnabled(), is(false));
    assertThat(updatedEntity.isStartTlsRequired(), is(true));
    assertThat(updatedEntity.isSslOnConnectEnabled(), is(false));
    assertThat(updatedEntity.isSslCheckServerIdentityEnabled(), is(true));
    assertThat(updatedEntity.isAstrumTrustStoreEnabled(), is(false));
  }

  @Test
  public void testDeleteEmailConfiguration() {
    EmailConfigurationData entity = createEmailConfigurationData(secret);

    dao.set(entity);
    EmailConfigurationData readEntity = dao.get().orElse(null);
    assertThat(readEntity, is(notNullValue()));

    dao.clear();
    readEntity = dao.get().orElse(null);
    assertThat(readEntity, is(nullValue()));
  }
}
