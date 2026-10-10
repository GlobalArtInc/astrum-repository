package org.globalart.astrum.ldap.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Records LDAP server changes in the audit log. Passwords are never recorded.
 */
@Component
public class LdapAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.ldap";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final LdapConfigurationChangedEvent event) {
    if (!isRecording()) {
      return;
    }
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    Map<String, Object> attributes = data.getAttributes();
    switch (event.getType()) {
      case CREATED -> data.setType(CREATED_TYPE);
      case UPDATED -> data.setType(UPDATED_TYPE);
      case DELETED -> data.setType(DELETED_TYPE);
      case REORDERED -> data.setType(CHANGED_TYPE);
    }
    LdapServer server = event.getServer();
    if (server == null) {
      data.setContext(SYSTEM_CONTEXT);
      attributes.put("order", string(event.getOrder()));
    }
    else {
      data.setContext(server.getName());
      attributes.put("id", server.getId());
      attributes.put("name", server.getName());
      attributes.put("url", server.getUrl());
      attributes.put("authScheme", server.getAuthScheme() != null ? server.getAuthScheme().name() : null);
      attributes.put("authUsername", server.getAuthUsername());
      attributes.put("groupType", server.getGroupType() != null ? server.getGroupType().name() : null);
    }
    record(data);
  }
}
