
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class TrustedSSLCertificateDAOTest
{

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(TrustedSSLCertificateDAO.class);

  private DataSession<?> session;

  private TrustedSSLCertificateDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(TrustedSSLCertificateDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testCreateReadUpdateDeleteOperations() {
    TrustedSSLCertificateData entity = new TrustedSSLCertificateData();
    entity.setAlias("keystorename");
    entity.setPem("somePemFormat");
    // Save the
    boolean saveResult = dao.save(entity);
    assertThat(saveResult, is(true));

    Optional<TrustedSSLCertificateData> readBack = dao.find(entity.getAlias());
    assertThat(readBack.isPresent(), is(true));
    assertThat(readBack.get().getAlias(), is(entity.getAlias()));
    assertThat(readBack.get().getPem(), is(entity.getPem()));

    entity.setPem("somePemFormat2");
    boolean updateResult = dao.save(entity);
    assertThat(updateResult, is(true));

    Optional<TrustedSSLCertificateData> updated = dao.find(entity.getAlias());
    assertThat(updated.isPresent(), is(true));
    assertThat(updated.get().getAlias(), is(entity.getAlias()));
    assertThat(updated.get().getPem(), is(entity.getPem()));

    List<TrustedSSLCertificateData> all = dao.findAll();
    assertThat(all.size(), is(1));

    boolean deleteResult = dao.delete(entity.getAlias());
    assertThat(deleteResult, is(true));

    Optional<TrustedSSLCertificateData> deleted = dao.find(entity.getAlias());
    assertThat(deleted.isPresent(), is(false));
  }
}
