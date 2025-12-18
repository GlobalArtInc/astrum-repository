
package org.globalart.astrum.security.internal;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.realm.RealmConfigurationChangedEvent;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Realm auditor.
 *
 * @since 3.1
 */
@Component
public class RealmAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.realm";

  private final RealmManager realmManager;

  @Autowired
  public RealmAuditor(final RealmManager realmManager) {
    this.realmManager = realmManager;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RealmConfigurationChangedEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(CHANGED_TYPE);
      data.setContext(SYSTEM_CONTEXT);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("realms", string(realmManager.getConfiguredRealmIds()));

      record(data);
    }
  }
}
