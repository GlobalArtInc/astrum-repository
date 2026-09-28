
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link TrustedSSLCertificateStore} implementation.
 */
@Component
@Qualifier("mybatis")
public class TrustedSSLCertificateStoreImpl
    extends ConfigStoreSupport<TrustedSSLCertificateDAO>
    implements TrustedSSLCertificateStore
{
  private final EventManager eventManager;

  @Autowired
  public TrustedSSLCertificateStoreImpl(
      final DataSessionSupplier sessionSupplier,
      final EventManager eventManager)
  {
    super(sessionSupplier);
    this.eventManager = checkNotNull(eventManager);
  }

  @Transactional
  @Override
  public Optional<TrustedSSLCertificate> find(final String alias) {
    Optional<TrustedSSLCertificateData> optional = dao().find(alias);
    return optional.map(data -> data);
  }

  @Transactional
  @Override
  public List<TrustedSSLCertificate> findAll() {
    List<TrustedSSLCertificateData> list = dao().findAll();

    return list.stream()
        .map(data -> (TrustedSSLCertificate) data)
        .toList();
  }

  @Transactional
  @Override
  public void save(final String alias, final String pem) {
    checkNotNull(alias);
    checkNotNull(pem);
    TrustedSSLCertificateData data = new TrustedSSLCertificateData(alias, pem);
    dao().save(data);
    postEvent(alias);
  }

  @Transactional
  @Override
  public void delete(final String alias) {
    checkNotNull(alias);
    dao().delete(alias);
    postEvent(alias);
  }

  private void postEvent(final String alias) {
    // trigger invalidation of TrustStoreImpl context
    eventManager.post((TrustedSSLCertificateDataEvent) () -> alias);
  }
}
