
package org.globalart.astrum.internal.ssl;

import java.security.cert.Certificate;
import java.security.cert.X509Certificate;
import java.util.HashMap;
import java.util.Map;

import javax.naming.ldap.LdapName;
import javax.naming.ldap.Rdn;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.ssl.CertificateCreatedEvent;
import org.globalart.astrum.ssl.CertificateDeletedEvent;
import org.globalart.astrum.ssl.CertificateEvent;

import com.google.common.base.Throwables;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * SSL certificate auditor.
 *
 * @since 3.1
 */
@Component
public class SslCertificateAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.sslcertificate";

  public SslCertificateAuditor() {
    registerType(CertificateCreatedEvent.class, CREATED_TYPE);
    registerType(CertificateDeletedEvent.class, DELETED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final CertificateEvent event) {
    if (isRecording()) {
      Certificate certificate = event.getCertificate();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("alias", event.getAlias());
      attributes.put("type", certificate.getType());

      if (certificate instanceof X509Certificate) {
        X509Certificate x509 = (X509Certificate) certificate;
        Map<String, String> rdns = parseLdapName(x509.getSubjectX500Principal().getName());
        data.setContext(rdns.get("CN"));
        attributes.putAll(rdns);
      }
      else {
        data.setContext(event.getAlias());
      }

      record(data);
    }
  }

  private static Map<String, String> parseLdapName(final String dn) {
    try {
      Map<String, String> result = new HashMap<>();
      LdapName ldapName = new LdapName(dn);
      for (Rdn rdn : ldapName.getRdns()) {
        result.put(rdn.getType(), rdn.getValue().toString());
      }
      return result;
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }
}
