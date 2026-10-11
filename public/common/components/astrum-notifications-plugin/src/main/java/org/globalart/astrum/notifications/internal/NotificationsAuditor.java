package org.globalart.astrum.notifications.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Records notification channel changes in the audit log. Bot tokens, webhook URLs and proxy passwords are never
 * recorded.
 */
@Component
public class NotificationsAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "system.notifications";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final NotificationChannelChangedEvent event) {
    if (!isRecording()) {
      return;
    }
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    switch (event.getType()) {
      case CREATED -> data.setType(CREATED_TYPE);
      case UPDATED -> data.setType(UPDATED_TYPE);
      case DELETED -> data.setType(DELETED_TYPE);
    }
    NotificationChannel channel = event.getChannel();
    data.setContext(channel.getName());
    Map<String, Object> attributes = data.getAttributes();
    attributes.put("id", channel.getId());
    attributes.put("name", channel.getName());
    attributes.put("type", channel.getType());
    attributes.put("enabled", string(channel.isEnabled()));
    attributes.put("target", channel.isTelegram() ? channel.getChatId() : channel.getTargetHost());
    attributes.put("repositories", string(channel.getRepositories()));
    attributes.put("events", string(channel.getEvents()));
    attributes.put("language", channel.getLanguage());
    attributes.put("proxyMode", channel.getProxyMode());
    if (channel.isCustomProxy()) {
      attributes.put("proxyType", channel.getProxyType());
      attributes.put("proxyHost", channel.getProxyHost());
      attributes.put("proxyPort", string(channel.getProxyPort()));
    }
    record(data);
  }
}
