package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.List;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * A notification channel as the user interface sees it. The secret (bot token or webhook URL) is never returned, only
 * a placeholder that keeps the stored value when it comes back.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class NotificationChannelXO
{
  private String id;

  private String name;

  private String type;

  private boolean enabled = true;

  private String chatId;

  private Long messageThreadId;

  private String secret;

  private String target;

  private List<String> repositories = new ArrayList<>();

  private List<String> events = new ArrayList<>();

  private String language;

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public String getChatId() {
    return chatId;
  }

  public void setChatId(final String chatId) {
    this.chatId = chatId;
  }

  public Long getMessageThreadId() {
    return messageThreadId;
  }

  public void setMessageThreadId(final Long messageThreadId) {
    this.messageThreadId = messageThreadId;
  }

  public String getSecret() {
    return secret;
  }

  public void setSecret(final String secret) {
    this.secret = secret;
  }

  /**
   * @return where the messages go: the chat id for Telegram, the webhook host otherwise; read only
   */
  public String getTarget() {
    return target;
  }

  public void setTarget(final String target) {
    this.target = target;
  }

  public List<String> getRepositories() {
    return repositories;
  }

  public void setRepositories(final List<String> repositories) {
    this.repositories = repositories != null ? repositories : new ArrayList<>();
  }

  public List<String> getEvents() {
    return events;
  }

  public void setEvents(final List<String> events) {
    this.events = events != null ? events : new ArrayList<>();
  }

  public String getLanguage() {
    return language;
  }

  public void setLanguage(final String language) {
    this.language = language;
  }
}
